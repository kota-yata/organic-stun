use thiserror::Error;

#[derive(Error, Debug)]
pub enum StunMessageError {
    #[error("Data too short to be a valid STUN message")]
    DataTooShort,

    #[error("Invalid magic cookie in STUN message")]
    InvalidMagicCookie,

    #[error("Invalid attribute length")]
    InvalidAttributeLength,

    #[error("Attribute data too short")]
    AttributeDataTooShort,

    #[error("Attribute length does not match data length")]
    AttributeLengthMismatch,

    #[error("Unsupported STUN request type")]
    UnsupportedRequestType,
}
