use crate::stun::message::{StunMessage, StunAttribute};
use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};

const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

pub fn handle_stun_request(request: &StunMessage, client_addr: &SocketAddr) -> Result<StunMessage, &'static str> {
  match request.message_type {
    0x0001 => handle_binding_request(request, client_addr),
    _ => Err("Unsupported STUN request type"),
  }
}

fn handle_binding_request(request: &StunMessage, client_addr: &SocketAddr) -> Result<StunMessage, &'static str> {
  let xor_mapped_address = create_xor_mapped_address(client_addr, &request.transaction_id)?;
  
  // Construct the Binding Response
  let response = StunMessage {
    message_type: 0x0101, // Binding Response
    length: 0, // to be calculated
    transaction_id: request.transaction_id,
    attributes: vec![xor_mapped_address],
  };

  Ok(response)
}

fn create_xor_mapped_address(client_addr: &SocketAddr, transaction_id: &[u8; 12]) -> Result<StunAttribute, &'static str> {
  match client_addr {
    SocketAddr::V4(addr) => {
      let ip = u32::from_ne_bytes(addr.ip().octets()) ^ STUN_MAGIC_COOKIE;
      let port = addr.port() ^ (STUN_MAGIC_COOKIE >> 16) as u16;

      let mut value = Vec::with_capacity(8);
      value.extend_from_slice(&[0, 1]); // IPv4 family
      value.extend_from_slice(&port.to_be_bytes());
      value.extend_from_slice(&ip.to_be_bytes());

      Ok(StunAttribute {
        attr_type: 0x0020, // XOR-MAPPED-ADDRESS
        length: value.len() as u16,
        value,
      })
    }
    SocketAddr::V6(addr) => {
      let ip = xor_ipv6_address(addr.ip(), transaction_id);
      let port = addr.port() ^ (STUN_MAGIC_COOKIE >> 16) as u16;

      let mut value = Vec::with_capacity(20);
      value.extend_from_slice(&[0, 2]); // IPv6 family
      value.extend_from_slice(&port.to_be_bytes());
      value.extend_from_slice(&ip);

      Ok(StunAttribute {
        attr_type: 0x0020, // XOR-MAPPED-ADDRESS
        length: value.len() as u16,
        value,
      })
    }
  }
}

fn xor_ipv6_address(addr: &Ipv6Addr, transaction_id: &[u8; 12]) -> [u8; 16] {
  let addr_segments = addr.segments();
  let mut xor_ip = [0u8; 16];

  for (i, &segment) in addr_segments.iter().enumerate() {
    let segment_bytes = if i < 4 {
      // XOR the first 64 bits with the magic cookie
      (segment as u16 ^ ((STUN_MAGIC_COOKIE >> (i * 16)) as u16)).to_be_bytes()
    } else {
      // XOR the rest with the transaction ID
      (segment as u16 ^ u16::from_be_bytes([transaction_id[2 * (i - 4)], transaction_id[2 * (i - 4) + 1]])).to_be_bytes()
    };
    xor_ip[2 * i] = segment_bytes[0];
    xor_ip[2 * i + 1] = segment_bytes[1];
  }

  xor_ip
}
