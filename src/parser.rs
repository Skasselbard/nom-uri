use super::*;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, character::*, combinator::*,
    error::ErrorKind, multi::*, number::complete::*, sequence::*, IResult,
};

macro_rules! many0_char_to_string {
    ($parser:expr) => {
        move |i| {
            let (_, position) = fold_many0($parser, 0, |mut pos: usize, _| {
                pos += 1;
                pos
            })(i)?;
            let (o, i) = i.split_at(position);
            let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
            Ok((i, o))
        }
    };
}
macro_rules! many1_char_to_string {
    ($parser:expr) => {
        move |i| {
            let (_, position) = fold_many1($parser, 0, |mut pos: usize, item| {
                pos += 1;
                pos
            })(i)?;
            let (o, i) = i.split_at(position);
            let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
            Ok((i, o))
        }
    };
}
// http://www.faqs.org/rfcs/rfc3986.html
// Appendix A.  Collected ABNF for URI

// URI           = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
// hier-part     = "//" authority path-abempty
//               / path-absolute
//               / path-rootless
//               / path-empty
// URI-reference = URI / relative-ref
// absolute-URI  = scheme ":" hier-part [ "?" query ]
// relative-ref  = relative-part [ "?" query ] [ "#" fragment ]
// relative-part = "//" authority path-abempty
//               / path-absolute
//               / path-noscheme
//               / path-empty
// scheme        = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
// authority     = [ userinfo "@" ] host [ ":" port ]
// userinfo      = *( unreserved / pct-encoded / sub-delims / ":" )
// host          = IP-literal / IPv4address / reg-name
// port          = *DIGIT
// IP-literal    = "[" ( IPv6address / IPvFuture  ) "]"
// IPvFuture     = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
// IPv6address   =                            6( h16 ":" ) ls32
//               /                       "::" 5( h16 ":" ) ls32
//               / [               h16 ] "::" 4( h16 ":" ) ls32
//               / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) ls32
//               / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) ls32
//               / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   ls32
//               / [ *4( h16 ":" ) h16 ] "::"              ls32
//               / [ *5( h16 ":" ) h16 ] "::"              h16
//               / [ *6( h16 ":" ) h16 ] "::"
// h16           = 1*4HEXDIG
// ls32          = ( h16 ":" h16 ) / IPv4address
// IPv4address   = dec-octet "." dec-octet "." dec-octet "." dec-octet
// dec-octet     = DIGIT                 ; 0-9
//               / %x31-39 DIGIT         ; 10-99
//               / "1" 2DIGIT            ; 100-199
//               / "2" %x30-34 DIGIT     ; 200-249
//               / "25" %x30-35          ; 250-255
// reg-name      = *( unreserved / pct-encoded / sub-delims )
// path          = path-abempty    ; begins with "/" or is empty
//               / path-absolute   ; begins with "/" but not "//"
//               / path-noscheme   ; begins with a non-colon segment
//               / path-rootless   ; begins with a segment
//               / path-empty      ; zero u8acters
// path-abempty  = *( "/" segment )
// path-absolute = "/" [ segment-nz *( "/" segment ) ]
// path-noscheme = segment-nz-nc *( "/" segment )
/// path-rootless = segment-nz *( "/" segment )
// fn path_rootless(i: &[u8]) -> IResult<&[u8], &str> {
//     let (nz, (i, segments)) = pair(segment,many0!(pair(char('/'),segment)))(i)?;
//     nz
//     // Ok()
// }
/// path-empty    = 0<pchar>
fn path_empty(i: &[u8]) -> IResult<&[u8], ()> {
    not(peek(pchar))(i)
}
/// segment       = *pchar
/// TODO:
fn segment(i: &[u8]) -> IResult<&[u8], &str> {
    many0_char_to_string!(pchar)(i)
}
/// segment-nz    = 1*pchar
/// TODO:
fn segment_nz(i: &[u8]) -> IResult<&[u8], &str> {
    many1_char_to_string!(pchar)(i)
}
/// segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
///               ; non-zero-length segment without any colon ":"
//TODO:
fn segment_nz_nc(i: &[u8]) -> IResult<&[u8], &str> {
    many1_char_to_string!(alt((
        alt((unreserved, pct_encoded)),
        alt((sub_delims, char('@')))
    )))(i)
}

/// pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"
fn pchar(i: &[u8]) -> IResult<&[u8], char> {
    alt((
        alt((unreserved, pct_encoded)),
        alt((sub_delims, one_of(":@"))),
    ))(i)
}
// query         = *( pchar / "/" / "?" )
fn query(i: &[u8]) -> IResult<&[u8], UriPart> {
    let (_, position) = fold_many0(alt((pchar, one_of("/?"))), 0, |mut pos: usize, item| {
        if peek(pct_encoded)(i.split_at(pos).1).is_ok() {
            pos += 3
        } else {
            pos += 1;
        }
        pos
    })(i)?;
    let (o, i) = i.split_at(position);
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    Ok((i, UriPart::Query(o)))
}
/// fragment      = *( pchar / "/" / "?" )
fn fragment(i: &[u8]) -> IResult<&[u8], UriPart> {
    let (i, o) = match query(i)? {
        (i, UriPart::Query(o)) => (i, o),
        _ => return Err(nom::Err::Error((i, ErrorKind::Many0))), //TODO: What error?
    };
    Ok((i, UriPart::Fragment(o)))
}
/// percentage encoded u32
/// pct-encoded   = "%" HEXDIG HEXDIG
fn pct_encoded(i: &[u8]) -> IResult<&[u8], char> {
    use core::char::from_u32;
    let (i, (high, low)) = preceded(char('%'), pair(hexdig, hexdig))(i)?;
    let hex_val = match hex_u32(&[high as u8, low as u8]) {
        Ok((_, o)) => o,
        Err(e) => match e {
            nom::Err::Incomplete(Needed) => return Err(nom::Err::Incomplete(Needed)),
            nom::Err::Error((_, e)) => return Err(nom::Err::Error((i, e))),
            nom::Err::Failure((_, e)) => return Err(nom::Err::Failure((i, e))),
        },
    };
    let o = match from_u32(hex_val) {
        Some(o) => o,
        None => return Err(nom::Err::Error((i, ErrorKind::HexDigit))),
    };
    Ok((i, o))
}
/// unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
fn unreserved(i: &[u8]) -> IResult<&[u8], char> {
    alt((alphanumeric, one_of("-._~")))(i)
}
/// reserved      = gen-delims / sub-delims
fn reserved(i: &[u8]) -> IResult<&[u8], char> {
    alt((gen_delims, sub_delims))(i)
}
/// gen-delims    = ":" / "/" / "?" / "#" / "[" / "]" / "@"
fn gen_delims(i: &[u8]) -> IResult<&[u8], char> {
    one_of(":/?#[]@")(i)
}
/// sub-delims    = "!" / "$" / "&" / "'" / "(" / ")"
///               / "*" / "+" / "," / ";" / "="
fn sub_delims(i: &[u8]) -> IResult<&[u8], char> {
    one_of("!$&'()*+,;=")(i)
}
fn alphanumeric(i: &[u8]) -> IResult<&[u8], char> {
    alt((alpha, digit))(i)
}
fn alpha(i: &[u8]) -> IResult<&[u8], char> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")(i)
}
fn digit(i: &[u8]) -> IResult<&[u8], char> {
    one_of("0123456789")(i)
}
fn hexdig(i: &[u8]) -> IResult<&[u8], char> {
    one_of("0123456789ABCDEFabcdef")(i)
}
fn is_hex_digit_u8(i: u8) -> bool {
    is_hex_digit(i as u8)
}
const pchar_no_pct: &[u8] =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~!$&'()*+,;=:@".as_bytes();
#[test]
fn fragment_test() {
    unsafe {
        assert_eq!(
            fragment(pchar_no_pct),
            Ok((
                &[][..],
                UriPart::Fragment(core::str::from_utf8_unchecked(&pchar_no_pct))
            ))
        )
    };
    assert_eq!(fragment(b"/?{"), Ok((&b"{"[..], UriPart::Fragment("/?"))));
    assert_eq!(
        fragment(b"%30%41#"),
        Ok((&b"#"[..], UriPart::Fragment("%30%41")))
    );
}
#[test]
fn pct_encoded_test() {
    assert_eq!(pct_encoded(b"%30*"), Ok((&b"*"[..], '0')));
    assert_eq!(pct_encoded(b"%41g"), Ok((&b"g"[..], 'A')));
    assert_eq!(
        pct_encoded(b"41"),
        Err(nom::Err::Error((&b"41"[..], ErrorKind::Char)))
    );
    assert_eq!(
        pct_encoded(b"%4"),
        Err(nom::Err::Error((&[][..], ErrorKind::OneOf)))
    );
}
