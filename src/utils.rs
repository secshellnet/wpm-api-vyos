use base64::DecodeError;

pub fn validate_key(input: &str) -> Result<(), DecodeError> {
    let regex = regex::Regex::new(r"^[A-Za-z0-9+/]{43}=$").unwrap();

    if !regex.is_match(input) {
        return Err(DecodeError::InvalidByte(0, 0));
    }

    let decoded = match base64::decode(input) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };

    decoded
}