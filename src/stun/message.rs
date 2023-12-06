use crate::stun::error::StunMessageError;

pub struct StunMessage {
  pub message_type: u16,
  pub length: u16,
  pub transaction_id: [u8; 12],
  pub attributes: Vec<StunAttribute>,
}

pub struct StunAttribute {
  pub attr_type: u16,
  pub length: u16,
  pub value: Vec<u8>,
}

impl StunMessage {
  /*
    Decode a STUN message from a byte array
       0                   1                   2                   3
       0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |0 0|     STUN Message Type     |         Message Length        |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |                         Magic Cookie                          |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |                                                               |
      |                     Transaction ID (96 bits)                  |
      |                                                               |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   */
  pub fn decode(data: &[u8]) -> Result<StunMessage, StunMessageError> {
    if data.len() < 20 {
      return Err(StunMessageError::DataTooShort);
    }

    let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    const STUN_MAGIC_COOKIE: u32 = 0x2112A442;
    if magic_cookie != STUN_MAGIC_COOKIE {
        return Err(StunMessageError::InvalidMagicCookie);
    }

    let message_type = ((data[0] as u16) << 8) | data[1] as u16;
    let length = ((data[2] as u16) << 8) | data[3] as u16;
    let mut transaction_id = [0u8; 12];
    transaction_id.copy_from_slice(&data[8..20]);

    let mut attributes = Vec::new();
    let mut offset = 20;
    while offset < data.len() {
      if offset + 4 > data.len() {
        return Err(StunMessageError::InvalidAttributeLength);
      }
      let attr = StunAttribute::decode(&data[offset..]).map_err(StunMessageError::from)?;
      offset += 4 + attr.length as usize;
      attributes.push(attr);
    }

    Ok(StunMessage {
      message_type,
      length,
      transaction_id,
      attributes,
    })
  }

  pub fn encode(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push((self.message_type >> 8) as u8);
    bytes.push(self.message_type as u8);
    bytes.push((self.length >> 8) as u8);
    bytes.push(self.length as u8);
    bytes.extend_from_slice(&self.transaction_id);
    for attr in &self.attributes {
      bytes.extend_from_slice(&attr.encode());
    }
    bytes
  }
}

impl StunAttribute {
  pub fn decode(data: &[u8]) -> Result<StunAttribute, StunMessageError> {
    if data.len() < 4 {
      return Err(StunMessageError::AttributeDataTooShort);
    }

    let attr_type = ((data[0] as u16) << 8) | data[1] as u16;
    let length = ((data[2] as u16) << 8) | data[3] as u16;

    if data.len() < (4 + length as usize) {
      return Err(StunMessageError::AttributeLengthMismatch);
    }

    let value = data[4..(4 + length as usize)].to_vec();

    Ok(StunAttribute {
      attr_type,
      length,
      value,
    })
  }

  pub fn encode(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push((self.attr_type >> 8) as u8);
    bytes.push(self.attr_type as u8);
    bytes.push((self.length >> 8) as u8);
    bytes.push(self.length as u8);
    bytes.extend_from_slice(&self.value);
    bytes
  }
}
