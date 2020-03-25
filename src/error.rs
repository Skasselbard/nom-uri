#[derive(PartialEq, Clone, Copy)]
pub enum Error {
    ParseError,
    ParseIncomplete,
    BufferToSmall,
    Conversion(core::str::Utf8Error),
    NoAuthority,
}

pub type ParserError<'a> = (&'a [u8], nom::error::ErrorKind);

pub fn nom_error_to_error(nom_error: nom::Err<ParserError>) -> Error {
    match nom_error {
        nom::Err::Error(e) | nom::Err::Failure(e) => match core::str::from_utf8(e.0) {
            Ok(_) => Error::ParseError,
            Err(utf8e) => Error::Conversion(utf8e),
        },
        nom::Err::Incomplete(_) => Error::ParseIncomplete,
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::ParseError => write!(f, "Could not parse input"),
            Error::ParseIncomplete => write!(f, "Incomplete parsing.",),
            Error::BufferToSmall => write!(f, "Output does not fit in buffer."),
            Error::Conversion(e) => write!(f, "Tried to convert non utf8 to string: {}", e),
            Error::NoAuthority => write!(
                f,
                "Tried to set authority field on an uri without authority."
            ),
        }
    }
}
