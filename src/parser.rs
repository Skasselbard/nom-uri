/// Based on grammar in http://www.faqs.org/rfcs/rfc3986.html
/// Appendix A.  Collected ABNF for URI
use super::*;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, error::ErrorKind,
    multi::*, number::complete::*, sequence::*, IResult,
};
macro_rules! fold_closure {
    ($i:ident, $pos:ident) => {
        if peek::<_, _, E, _>(pct_encoded)($i.split_at($pos).1).is_ok() {
            $pos + 3
        } else {
            $pos + 1
        }
    };
}
// applies a parser that returns a &str m to n times and returns the result as &str
fn many_str_m_n<'a, E, F>(
    m: usize,
    n: usize,
    f: F,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a str, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], &str, E> + Copy,
    E: nom::error::ParseError<&'a [u8]>,
{
    move |i| {
        let (_, position) = match fold_many_m_n(m, n, f, 0, |mut pos: usize, segment| {
            pos += segment.len();
            pos
        })(i)
        {
            Ok((i, o)) => (i, o),
            Err(e) => return Err(e),
        };
        Ok(split_input_to_str(i, position))
    }
}
/// Split the byte array in two.
///
/// The left part is considered parsed and the right part is considered unparsed.
/// The parsed part is reinterpreted as &str and returned with the rest of the original array.
/// The reinterpretation is *unsafe*. Do not use this function on unparsed input.
fn split_input_to_str(input: &[u8], position: usize) -> (&[u8], &str) {
    // TODO: use the safe cast?
    let (o, i) = input.split_at(position); // one colon
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    (i, o)
}
/// ```abnf
/// URI           = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
/// absolute-URI  = scheme ":" hier-part [ "?" query ]
/// absolute uri does not matter for parsing and can be generated by omitting the fragment
/// ```
pub fn uri<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], Uri, E> {
    let (i, (s, (a, p), q, f)) = tuple((
        scheme,
        preceded(char(':'), hier_part),
        opt(preceded(char('?'), query)),
        opt(preceded(char('#'), fragment)),
    ))(i)?;
    Ok((
        i,
        Uri {
            scheme: s,
            authority: a,
            path: p,
            query: q,
            fragment: f,
        },
    ))
}

/// ```abnf
/// hier-part     = "//" authority path-abempty
///               / path-absolute
///               / path-rootless
///               / path-empty
/// ```
fn hier_part<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], (Option<Authority>, Path), E> {
    match pair::<_, _, _, E, _, _>(preceded(tag("//"), authority), path_abempty)(i) {
        Ok((i, (a, p))) => Ok((i, (Some(a), p))),
        Err(e) => {
            let (i, p) = alt((path_absolute, path_rootless, path_empty))(i)?;
            Ok((i, (None, p)))
        }
    }
}
/// ```abnf
/// URI-reference = URI / relative-ref
/// ```
#[allow(unused)]
fn uri_reference<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], UriReference, E> {
    match uri::<E>(i) {
        Ok((rest, o)) => Ok((rest, UriReference::Uri(o))),
        _ => {
            let (rest, (a, p, q, f)) = relative_ref(i)?;
            Ok((
                rest,
                UriReference::Reference(Reference {
                    authority: a,
                    path: p,
                    query: q,
                    fragment: f,
                }),
            ))
        }
    }
}
/// ```abnf
/// relative-ref  = relative-part [ "?" query ] [ "#" fragment ]
/// ```
fn relative_ref<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], (Option<Authority>, Path, Option<Query>, Option<Fragment>), E> {
    let (i, ((a, p), q, f)) = tuple((
        relative_part,
        opt(preceded(char('?'), query)),
        opt(preceded(char('#'), fragment)),
    ))(i)?;
    Ok((i, (a, p, q, f)))
}
/// ```abnf
/// relative-part = "//" authority path-abempty
///               / path-absolute
///               / path-noscheme
///               / path-empty
/// ```
fn relative_part<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], (Option<Authority>, Path), E> {
    match pair::<_, _, _, E, _, _>(preceded(tag("//"), authority), path_abempty)(i) {
        Ok((i, (a, p))) => Ok((i, (Some(a), p))),
        _ => {
            let (i, p) = alt((path_absolute, path_noscheme, path_empty))(i)?;
            Ok((i, (None, p)))
        }
    }
}
/// ```abnf
/// scheme        = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
/// ```
pub fn scheme<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (_, (_, position)) = pair(
        alpha,
        fold_many0(
            alt((alphanumeric, one_of("+-."))),
            0,
            |mut pos: usize, _| {
                pos = fold_closure!(i, pos);
                pos
            },
        ),
    )(i)?;
    Ok(split_input_to_str(i, position + 1)) // one alpha at the start
}
/// ```abnf
/// authority     = [ userinfo "@" ] host [ ":" port ]
/// ```
pub(crate) fn authority<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Authority, E> {
    let (rest, (user_info, hos_t, por_t)) = tuple((
        opt(terminated(userinfo, char('@'))),
        host,
        opt(preceded(char(':'), port)),
    ))(i)?;
    let auth = Authority {
        userinfo: user_info,
        host: hos_t,
        port: por_t.flatten(),
    };
    Ok((rest, auth))
}
/// ```abnf
/// userinfo      = *( unreserved / pct-encoded / sub-delims / ":" )
/// ```
pub fn userinfo<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], &str, E> {
    let (_, position) = fold_many1(
        alt((unreserved, pct_encoded, sub_delims, char(':'))),
        0,
        |mut pos: usize, _| {
            pos = fold_closure!(i, pos);
            pos
        },
    )(i)?;
    Ok(split_input_to_str(i, position))
}
/// ```abnf
/// host          = IP-literal / IPv4address / reg-name
/// ```
pub fn host<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], Host, E> {
    alt((ip_literal, ip_v4_address, reg_name))(i)
}
/// ```abnf
/// port          = *DIGIT
/// ```
pub fn port<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Option<&str>, E> {
    let (rest, o) = digit0(i)?;
    if o.len() == 0 {
        // port can be empty
        return Ok((i, None));
    };
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    match u16::from_str_radix(o, 10) {
        // u16 max_value() = port_max => no extra value check
        Err(_) => return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::Digit))),
        Ok(_) => {}
    };
    Ok((rest, Some(o)))
}
/// ```abnf
/// IP-literal    = "[" ( IPv6address / IPvFuture  ) "]"
/// ```
fn ip_literal<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], Host, E> {
    let (rest, (_, ip, _)) = tuple((char('['), alt((ip_v6_address, ip_v_future)), char(']')))(i)?;
    Ok((rest, ip))
}
/// ```abnf
/// IPvFuture     = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
/// Unimplemented!
/// ```
fn ip_v_future<'a, E: nom::error::ParseError<&'a [u8]>>(
    _i: &'a [u8],
) -> IResult<&'a [u8], Host, E> {
    unimplemented!();
}
/// ```abnf
/// IPv6address   =                            6( h16 ":" ) (ls32 / IPv4address)
///               /                       "::" 5( h16 ":" ) (ls32 / IPv4address)
///               / [               h16 ] "::" 4( h16 ":" ) (ls32 / IPv4address)
///               / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) (ls32 / IPv4address)
///               / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) (ls32 / IPv4address)
///               / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   (ls32 / IPv4address)
///               / [ *4( h16 ":" ) h16 ] "::"              (ls32 / IPv4address)
///               / [ *5( h16 ":" ) h16 ] "::"              h16
/// ```
pub fn ip_v6_address<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Host, E> {
    let (i, o) = alt((ip_v6_long, ip_v6_short))(i)?;
    Ok((i, Host::V6(o)))
}
/// ```abnf
/// /                       "::" 5( h16 ":" ) (ls32 / IPv4address)
/// / [               h16 ] "::" 4( h16 ":" ) (ls32 / IPv4address)
/// / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) (ls32 / IPv4address)
/// / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) (ls32 / IPv4address)
/// / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   (ls32 / IPv4address)
/// / [ *4( h16 ":" ) h16 ] "::"              (ls32 / IPv4address)
/// / [ *5( h16 ":" ) h16 ] "::"              h16
/// / [ *6( h16 ":" ) h16 ] "::"
/// ```
fn ip_v6_short<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (i, (left_colons, right_colons)) = alt((
        separated_pair(
            opt(pair(many_str_m_n(0, 0, h16_colon), many_str_m_n(0, 0, h16))),
            tag("::"),
            pair(many_str_m_n(4, 4, h16_colon), ip_v6_end),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 0, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(4, 4, h16_colon), ip_v6_end),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 1, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(3, 3, h16_colon), ip_v6_end),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 2, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(2, 2, h16_colon), ip_v6_end),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 3, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(1, 1, h16_colon), ip_v6_end),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 4, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(0, 0, h16_colon), ip_v6_end),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 5, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(0, 0, h16_colon), h16),
        ),
        separated_pair(
            opt(pair(many_str_m_n(0, 6, h16_colon), h16)),
            tag("::"),
            pair(many_str_m_n(0, 0, h16_colon), many_str_m_n(0, 0, h16)),
        ),
    ))(i)?;
    let mut position = match left_colons {
        Some((l, r)) => l.len() + r.len(),
        None => 0,
    };
    position += right_colons.0.len();
    position += right_colons.1.len();
    Ok(split_input_to_str(i, position))
}
/// ```abnf
/// 6( h16 ":" ) (ls32 / IPv4address)
/// ```
fn ip_v6_long<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    many_str_m_n(6, 6, ip_v6_end)(i)
}
/// (ls32 / IPv4address)
fn ip_v6_end<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    match opt(ip_v4_address)(i)? {
        (rest, Some(Host::V4(o))) => Ok((rest, o)),
        (_, None) => Ok(ls32(i)?),
        _ => unreachable!(),
    }
}
/// ```abnf
/// ( h16 ":" )
/// ```
fn h16_colon<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (_, (o1, _)) = pair(h16, char(':'))(i)?;
    Ok(split_input_to_str(i, o1.len() + 1)) // one colon
}
/// ```abnf
/// h16           = 1*4HEXDIG
/// 16 bits of address represented in hexadecimal
/// ```
fn h16<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (rest, o) = hex_digit1(i)?;
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    match u16::from_str_radix(o, 16) {
        // u16 max_value() = FFFF => no extra value check
        Err(_) => return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::Digit))),
        _ => {}
    };
    Ok((rest, o))
}
/// ```abnf
/// ls32          = ( h16 ":" h16 )
/// least-significant 32 bits of address
/// According to rfc3986 this part can also be an IPv4Address,
/// but we parse that option separatly in ip_v6_end().
/// ```
fn ls32<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (_, (o1, _, o2)) = tuple((h16, char(':'), h16))(i)?;
    Ok(split_input_to_str(i, o1.len() + o2.len() + 1)) // one colon
}
/// ```abnf
/// IPv4address   = dec-octet "." dec-octet "." dec-octet "." dec-octet
/// ```
pub fn ip_v4_address<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Host, E> {
    let (_, (o1, _, o2, _, o3, _, o4)) = tuple((
        dec_octet,
        char('.'),
        dec_octet,
        char('.'),
        dec_octet,
        char('.'),
        dec_octet,
    ))(i)?;
    let (i, o) = split_input_to_str(i, o1.len() + o2.len() + o3.len() + o4.len() + 3); // three dots
    Ok((i, Host::V4(o)))
}

/// ```abnf
/// dec-octet     = DIGIT                 ; 0-9
///               / %x31-39 DIGIT         ; 10-99
///               / "1" 2DIGIT            ; 100-199
///               / "2" %x30-34 DIGIT     ; 200-249
///               / "25" %x30-35          ; 250-255
/// ```
fn dec_octet<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (rest, o) = digit1(i)?;
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    match u8::from_str_radix(o, 10) {
        // u8 max_value() = 255 => no extra value check
        Err(_) => return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::Digit))),
        _ => {}
    };
    Ok((rest, o))
}
/// ```abnf
/// reg-name      = *( unreserved / pct-encoded / sub-delims )
/// ```
fn reg_name<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], Host, E> {
    let (_, position) = fold_many1(
        alt((unreserved, pct_encoded, sub_delims)),
        0,
        |mut pos: usize, _| {
            pos = fold_closure!(i, pos);
            pos
        },
    )(i)?;
    let (i, o) = split_input_to_str(i, position);
    Ok((i, Host::RegistryName(o)))
}
/// ```abnf
/// path          = path-abempty    ; begins with "/" or is empty
///               / path-absolute   ; begins with "/" but not "//"
///               / path-noscheme   ; begins with a non-colon segment
///               / path-rootless   ; begins with a segment
///               / path-empty      ; zero characters
/// ```
pub(crate) fn path<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Path, E> {
    alt((
        path_absolute,
        path_noscheme,
        path_rootless,
        path_abempty,
        path_empty,
    ))(i)
}
/// ```abnf
/// path-absolute = "/" [ segment-nz *( "/" segment ) ]
/// ```
fn path_absolute<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Path, E> {
    let (_, (_, segments)) = pair(char('/'), opt(path_rootless))(i)?;
    let segments = match segments {
        Some(Path::Rootless(path)) => path,
        None => "",
        _ => unreachable!(),
    };
    let (i, o) = split_input_to_str(i, 1 + segments.len());
    Ok((i, Path::Absolute(o)))
}
/// ```abnf
/// path-noscheme = segment-nz-nc *( "/" segment )
/// ```
fn path_noscheme<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Path, E> {
    let (_, (nz, segments)) = pair(segment_nz_nc, path_abempty)(i)?;
    let segments = match segments {
        Path::AbEmpty(path) => path,
        _ => unreachable!(),
    };
    let (i, o) = split_input_to_str(i, nz.len() + segments.len());
    Ok((i, Path::NoScheme(o)))
}
/// ```abnf
/// path-rootless = segment-nz *( "/" segment )
/// ```
fn path_rootless<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Path, E> {
    let (_, (nz, segments)) = pair(segment_nz, path_abempty)(i)?;
    let segments = match segments {
        Path::AbEmpty(path) => path,
        _ => unreachable!(),
    };
    let (i, o) = split_input_to_str(i, nz.len() + segments.len());
    Ok((i, Path::Rootless(o)))
}
/// ```abnf
/// path-abempty  = *( "/" segment )
/// ```
fn path_abempty<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Path, E> {
    let (_, position) = fold_many0(
        preceded(char('/'), cut(segment)),
        0,
        |mut pos: usize, segment| {
            pos += 1 + segment.len(); //add one for the '/'
            pos
        },
    )(i)?;
    let (i, o) = split_input_to_str(i, position);
    Ok((i, Path::AbEmpty(o)))
}

/// ```abnf
/// path-empty    = 0<pchar>
/// ```
fn path_empty<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], Path, E> {
    not(peek(pchar))(i)?;
    Ok((i, Path::Empty))
}
/// ```abnf
/// segment       = *pchar
/// ```
fn segment<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (_, position) = fold_many0(pchar, 0, |mut pos: usize, _| {
        pos = fold_closure!(i, pos);
        pos
    })(i)?;
    Ok(split_input_to_str(i, position))
}
/// ```abnf
/// segment-nz    = 1*pchar
/// ```
fn segment_nz<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], &str, E> {
    let (_, position) = fold_many1(pchar, 0, |mut pos: usize, _| {
        pos = fold_closure!(i, pos);
        pos
    })(i)?;
    Ok(split_input_to_str(i, position))
}
/// ```abnf
/// segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
/// non-zero-length segment without any colon ":"
/// ```
fn segment_nz_nc<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], &str, E> {
    let (_, position) = fold_many1(
        alt((unreserved, pct_encoded, sub_delims, char('@'))),
        0,
        |mut pos: usize, _| {
            pos = fold_closure!(i, pos);
            pos
        },
    )(i)?;
    Ok(split_input_to_str(i, position))
}
/// ```abnf
/// pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
fn pchar<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    alt((unreserved, pct_encoded, sub_delims, one_of(":@")))(i)
}
/// ```abnf
/// query         = *( pchar / "/" / "?" )
/// ```
pub(crate) fn query<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Query, E> {
    let (_, position) = fold_many0(alt((pchar, one_of("/?"))), 0, |mut pos: usize, _| {
        pos = fold_closure!(i, pos);
        pos
    })(i)?;
    let (i, o) = split_input_to_str(i, position);
    Ok((i, Query(o)))
}
/// ```abnf
/// fragment      = *( pchar / "/" / "?" )
/// ```
pub(crate) fn fragment<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Fragment, E> {
    #[allow(unreachable_patterns)]
    let (i, o) = match query(i)? {
        (i, Query(o)) => (i, o),
        _ => unreachable!(),
    };
    Ok((i, Fragment(o)))
}
/// ```abnf
/// percentage encoded u32
/// pct-encoded   = "%" HEXDIG HEXDIG
/// ```
fn pct_encoded<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    use core::char::from_u32;
    let (i, (high, low)) = preceded(char('%'), pair(hexdig, hexdig))(i)?;
    let hex_val = match hex_u32(&[high as u8, low as u8]) {
        Ok((_, o)) => o,
        Err(e) => match e {
            nom::Err::Incomplete(needed) => return Err(nom::Err::Incomplete(needed)),
            nom::Err::Error((_, e)) => return Err(nom::Err::Failure(E::from_error_kind(i, e))),
            nom::Err::Failure((_, e)) => return Err(nom::Err::Failure(E::from_error_kind(i, e))),
        },
    };
    let o = match from_u32(hex_val) {
        Some(o) => o,
        None => return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::HexDigit))),
    };
    Ok((i, o))
}
/// ```abnf
/// unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
/// ```
fn unreserved<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    alt((alphanumeric, one_of("-._~")))(i)
}
/// ```abnf
/// reserved      = gen-delims / sub-delims
/// ```
#[allow(unused)]
fn reserved<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    alt((gen_delims, sub_delims))(i)
}
/// ```abnf
/// gen-delims    = ":" / "/" / "?" / "#" / "[" / "]" / "@"
/// ```
#[allow(unused)]
fn gen_delims<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    one_of(":/?#[]@")(i)
}
/// ```abnf
/// sub-delims    = "!" / "$" / "&" / "'" / "(" / ")"
///               / "*" / "+" / "," / ";" / "="
/// ```
fn sub_delims<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    one_of("!$&'()*+,;=")(i)
}
fn alphanumeric<'a, E: nom::error::ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], char, E> {
    alt((alpha, digit))(i)
}
fn alpha<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")(i)
}
fn digit<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    one_of("0123456789")(i)
}
fn hexdig<'a, E: nom::error::ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], char, E> {
    one_of("0123456789ABCDEFabcdef")(i)
}
#[allow(unused)]
const PCHAR_NO_PCT: &[u8] =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~!$&'()*+,;=:@".as_bytes();
#[test]
fn port_test() {
    assert_eq!(port::<(&[u8], ErrorKind)>(b""), Ok((&b""[..], None)));
}
#[test]
fn ip_v4_test() {
    assert_eq!(
        ip_v4_address(b"24.4.34"),
        Err(nom::Err::Error((&b""[..], ErrorKind::Char)))
    );
    assert_eq!(
        ip_v4_address(b"256.24.4.34"),
        Err(nom::Err::Error((&b"256.24.4.34"[..], ErrorKind::Digit)))
    );
    assert_eq!(
        ip_v4_address::<(&[u8], ErrorKind)>(b"255.255.255.255.255"),
        Ok((&b".255"[..], Host::V4("255.255.255.255")))
    );
    assert_eq!(
        ip_v4_address::<(&[u8], ErrorKind)>(b"255.255.255.255"),
        Ok((&b""[..], Host::V4("255.255.255.255")))
    );
    assert_eq!(
        ip_v4_address::<(&[u8], ErrorKind)>(b"0.0.0.0"),
        Ok((&b""[..], Host::V4("0.0.0.0")))
    );
}
#[test]
fn path_absolute_test() {
    assert_eq!(
        path_absolute(b"abc/def//"),
        Err(nom::Err::Error((&b"abc/def//"[..], ErrorKind::Char)))
    );
    assert_eq!(
        path_absolute::<(&[u8], ErrorKind)>(b"/abc/def//"),
        Ok((&b""[..], Path::Absolute("/abc/def//")))
    );
}
#[test]
fn path_rootless_test() {
    assert_eq!(
        path_rootless(b"/abc/def//"),
        Err(nom::Err::Error((&b"/abc/def//"[..], ErrorKind::Many1)))
    );
    assert_eq!(
        path_rootless::<(&[u8], ErrorKind)>(b"abc/def//"),
        Ok((&b""[..], Path::Rootless("abc/def//")))
    );
}
#[test]
fn path_abempty_test() {
    assert_eq!(
        path_abempty::<(&[u8], ErrorKind)>(b"/abc/def//"),
        Ok((&[][..], Path::AbEmpty("/abc/def//")))
    );
    assert_eq!(
        path_abempty::<(&[u8], ErrorKind)>(b"abc/def//"),
        Ok((&b"abc/def//"[..], Path::AbEmpty("")))
    );
}
#[test]
fn fragment_test() {
    unsafe {
        assert_eq!(
            fragment::<(&[u8], ErrorKind)>(PCHAR_NO_PCT),
            Ok((
                &[][..],
                Fragment(core::str::from_utf8_unchecked(&PCHAR_NO_PCT))
            ))
        )
    };
    assert_eq!(
        fragment::<(&[u8], ErrorKind)>(b"/?{"),
        Ok((&b"{"[..], Fragment("/?")))
    );
    assert_eq!(
        fragment::<(&[u8], ErrorKind)>(b"%30%41#"),
        Ok((&b"#"[..], Fragment("%30%41")))
    );
}
#[test]
fn pct_encoded_test() {
    assert_eq!(
        pct_encoded::<(&[u8], ErrorKind)>(b"%30*"),
        Ok((&b"*"[..], '0'))
    );
    assert_eq!(
        pct_encoded::<(&[u8], ErrorKind)>(b"%41g"),
        Ok((&b"g"[..], 'A'))
    );
    assert_eq!(
        pct_encoded(b"41"),
        Err(nom::Err::Error((&b"41"[..], ErrorKind::Char)))
    );
    assert_eq!(
        pct_encoded(b"%4"),
        Err(nom::Err::Error((&[][..], ErrorKind::OneOf)))
    );
}
