#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error<'e> {
    ParseError(&'e str),
    ParseIncomplete(nom::Needed),
    BufferToSmall,
    Conversion(core::str::Utf8Error),
    NoAuthority,
}

pub type ParserError<'a> = (&'a [u8], nom::error::ErrorKind);

pub fn nom_error_to_error(nom_error: nom::Err<ParserError>) -> Error {
    match nom_error {
        nom::Err::Error(e) | nom::Err::Failure(e) => {
            let parsed = match core::str::from_utf8(e.0) {
                Ok(parsed) => parsed,
                Err(utf8e) => return Error::Conversion(utf8e),
            };
            Error::ParseError(parsed)
        }
        nom::Err::Incomplete(needed) => Error::ParseIncomplete(needed),
    }
}

// impl<'e> core::fmt::Debug for Error<'e> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         match self {
//             Error::ParseError(string) => write!(f, "Could not parse input: {}", string),
//             Error::ParseIncomplete(needed) => write!(
//                 f,
//                 "Incomplete parsing. Additional needed symbols: {:?}",
//                 needed
//             ),
//             Error::BufferToSmall => write!(f, "Output does not fit in buffer."),
//             Error::Conversion(e) => write!(f, "Tried to convert non utf8 to string: {}", e),
//             Error::NoAuthority => write!(
//                 f,
//                 "Tried to set authority field on an uri without authority."
//             ),
//         }
//     }
// }
