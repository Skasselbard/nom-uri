/// Based on grammar in http://www.faqs.org/rfcs/rfc3986.html
/// Appendix A.  Collected ABNF for URI
use super::*;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, character::*, combinator::*,
    error::ErrorKind, multi::*, number::complete::*, sequence::*, IResult,
};
macro_rules! fold_closure {
    ($i:ident, $pos:ident) => {
        if peek(pct_encoded)($i.split_at($pos).1).is_ok() {
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
fn split_input_to_str(input: &[u8], position: usize) -> (&[u8], &str) {
    let (o, i) = input.split_at(position); // one colon
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    (i, o)
}
/// ```ignore
/// URI           = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
/// absolute-URI  = scheme ":" hier-part [ "?" query ]
/// absolute uri does not matter for parsing and can be generated by omitting the fragment
/// ```
fn uri(i: &[u8]) -> IResult<&[u8], Uri> {
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

/// ```ignore
/// hier-part     = "//" authority path-abempty
///               / path-absolute
///               / path-rootless
///               / path-empty
/// ```
fn hier_part(i: &[u8]) -> IResult<&[u8], (Authority, Path)> {
    preceded(
        tag("//"),
        pair(
            authotity,
            alt((path_abempty, path_absolute, path_rootless, path_empty)),
        ),
    )(i)
}
/// ```ignore
/// URI-reference = URI / relative-ref
/// ```
fn uri_reference(i: &[u8]) -> IResult<&[u8], UriReference> {
    match uri(i) {
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
/// ```ignore
/// relative-ref  = relative-part [ "?" query ] [ "#" fragment ]
/// ```
fn relative_ref(i: &[u8]) -> IResult<&[u8], (Authority, Path, Option<Query>, Option<Fragment>)> {
    let (i, ((a, p), q, f)) = tuple((
        relative_part,
        opt(preceded(char('?'), query)),
        opt(preceded(char('#'), fragment)),
    ))(i)?;
    Ok((i, (a, p, q, f)))
}
/// ```ignore
/// relative-part = "//" authority path-abempty
///               / path-absolute
///               / path-noscheme
///               / path-empty
/// ```
fn relative_part(i: &[u8]) -> IResult<&[u8], (Authority, Path)> {
    preceded(
        tag("//"),
        pair(
            authotity,
            alt((path_abempty, path_absolute, path_noscheme, path_empty)),
        ),
    )(i)
}
/// ```ignore
/// scheme        = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
/// ```
fn scheme(i: &[u8]) -> IResult<&[u8], &str> {
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
/// ```ignore
/// authority     = [ userinfo "@" ] host [ ":" port ]
/// ```
fn authotity(i: &[u8]) -> IResult<&[u8], Authority> {
    let (_, (user_info, hos_t, por_t)) =
        tuple((opt(userinfo), host, opt(preceded(char(':'), port))))(i)?;
    let auth = Authority {
        userinfo: user_info,
        host: hos_t,
        port: por_t.flatten(),
    };
    Ok((i, auth))
}
/// ```ignore
/// userinfo      = *( unreserved / pct-encoded / sub-delims / ":" )
/// ```
fn userinfo(i: &[u8]) -> IResult<&[u8], &str> {
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
/// ```ignore
/// host          = IP-literal / IPv4address / reg-name
/// ```
fn host(i: &[u8]) -> IResult<&[u8], Host> {
    alt((ip_literal, ip_v4_address, reg_name))(i)
}
/// ```ignore
/// port          = *DIGIT
/// ```
fn port(i: &[u8]) -> IResult<&[u8], Option<u16>> {
    let (rest, o) = digit0(i)?;
    if o.len() == 0 {
        // port can be empty
        return Ok((i, None));
    };
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    let o = match u16::from_str_radix(o, 10) {
        // u16 max_value() = port_max => no extra value check
        Err(_) => return Err(nom::Err::Error((i, ErrorKind::Digit))),
        Ok(port) => port,
    };
    Ok((rest, Some(o)))
}
/// ```ignore
/// IP-literal    = "[" ( IPv6address / IPvFuture  ) "]"
/// ```
fn ip_literal(i: &[u8]) -> IResult<&[u8], Host> {
    let (rest, (_, ip, _)) = tuple((char('['), alt((ip_v6_address, ip_v_future)), char(']')))(i)?;
    Ok((rest, ip))
}
/// ```ignore
/// IPvFuture     = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
/// Unimplemented!
/// ```
fn ip_v_future(i: &[u8]) -> IResult<&[u8], Host> {
    unimplemented!();
}
/// ```ignore
/// IPv6address   =                            6( h16 ":" ) (ls32 / IPv4address)
///               /                       "::" 5( h16 ":" ) (ls32 / IPv4address)
///               / [               h16 ] "::" 4( h16 ":" ) (ls32 / IPv4address)
///               / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) (ls32 / IPv4address)
///               / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) (ls32 / IPv4address)
///               / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   (ls32 / IPv4address)
///               / [ *4( h16 ":" ) h16 ] "::"              (ls32 / IPv4address)
///               / [ *5( h16 ":" ) h16 ] "::"              h16
/// ```
fn ip_v6_address(i: &[u8]) -> IResult<&[u8], Host> {
    let (i, o) = alt((ip_v6_long, ip_v6_short))(i)?;
    Ok((i, Host::V6(o)))
}
/// ```ignore
/// /                       "::" 5( h16 ":" ) (ls32 / IPv4address)
/// / [               h16 ] "::" 4( h16 ":" ) (ls32 / IPv4address)
/// / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) (ls32 / IPv4address)
/// / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) (ls32 / IPv4address)
/// / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   (ls32 / IPv4address)
/// / [ *4( h16 ":" ) h16 ] "::"              (ls32 / IPv4address)
/// / [ *5( h16 ":" ) h16 ] "::"              h16
/// / [ *6( h16 ":" ) h16 ] "::"
/// ```
fn ip_v6_short(i: &[u8]) -> IResult<&[u8], &str> {
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
/// ```ignore
/// 6( h16 ":" ) (ls32 / IPv4address)
/// ```
fn ip_v6_long(i: &[u8]) -> IResult<&[u8], &str> {
    many_str_m_n(6, 6, ip_v6_end)(i)
}
/// (ls32 / IPv4address)
fn ip_v6_end(i: &[u8]) -> IResult<&[u8], &str> {
    match opt(ip_v4_address)(i)? {
        (rest, Some(Host::V4(o))) => Ok((rest, o)),
        (_, None) => Ok(ls32(i)?),
        _ => unreachable!(),
    }
}
/// ```ignore
/// ( h16 ":" )
/// ```
fn h16_colon(i: &[u8]) -> IResult<&[u8], &str> {
    let (_, (o1, _)) = pair(h16, char(':'))(i)?;
    Ok(split_input_to_str(i, o1.len() + 1)) // one colon
}
/// ```ignore
/// h16           = 1*4HEXDIG
/// 16 bits of address represented in hexadecimal
/// ```
fn h16(i: &[u8]) -> IResult<&[u8], &str> {
    let (rest, o) = hex_digit1(i)?;
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    match u16::from_str_radix(o, 16) {
        // u16 max_value() = FFFF => no extra value check
        Err(_) => return Err(nom::Err::Error((i, ErrorKind::Digit))),
        _ => {}
    };
    Ok((rest, o))
}
/// ```ignore
/// ls32          = ( h16 ":" h16 )
/// least-significant 32 bits of address
/// According to rfc3986 this part can also be an IPv4Address,
/// but we parse that option separatly in ip_v6_end().
/// ```
fn ls32(i: &[u8]) -> IResult<&[u8], &str> {
    let (_, (o1, _, o2)) = tuple((h16, char(':'), h16))(i)?;
    Ok(split_input_to_str(i, o1.len() + o2.len() + 1)) // one colon
}
/// ```ignore
/// IPv4address   = dec-octet "." dec-octet "." dec-octet "." dec-octet
/// ```
fn ip_v4_address(i: &[u8]) -> IResult<&[u8], Host> {
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

/// ```ignore
/// dec-octet     = DIGIT                 ; 0-9
///               / %x31-39 DIGIT         ; 10-99
///               / "1" 2DIGIT            ; 100-199
///               / "2" %x30-34 DIGIT     ; 200-249
///               / "25" %x30-35          ; 250-255
/// ```
fn dec_octet(i: &[u8]) -> IResult<&[u8], &str> {
    let (rest, o) = digit1(i)?;
    let o = unsafe { core::str::from_utf8_unchecked(o) }; // already parsed -> cannot fail
    match u8::from_str_radix(o, 10) {
        // u8 max_value() = 255 => no extra value check
        Err(_) => return Err(nom::Err::Error((i, ErrorKind::Digit))),
        _ => {}
    };
    Ok((rest, o))
}
/// ```ignore
/// reg-name      = *( unreserved / pct-encoded / sub-delims )
/// ```
fn reg_name(i: &[u8]) -> IResult<&[u8], Host> {
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
/// ```ignore
/// path          = path-abempty    ; begins with "/" or is empty
///               / path-absolute   ; begins with "/" but not "//"
///               / path-noscheme   ; begins with a non-colon segment
///               / path-rootless   ; begins with a segment
///               / path-empty      ; zero u8acters
/// ```
fn path(i: &[u8]) -> IResult<&[u8], Path> {
    alt((
        path_abempty,
        path_absolute,
        path_noscheme,
        path_rootless,
        path_empty,
    ))(i)
}
/// ```ignore
/// path-absolute = "/" [ segment-nz *( "/" segment ) ]
/// ```
fn path_absolute(i: &[u8]) -> IResult<&[u8], Path> {
    let (rest, (_, segments)) = pair(char('/'), opt(path_rootless))(i)?;
    let segments = match segments {
        Some(Path::Rootless(path)) => path,
        None => "",
        _ => unreachable!(),
    };
    let (i, o) = split_input_to_str(i, 1 + segments.len());
    Ok((i, Path::Absolute(o)))
}
/// ```ignore
/// path-noscheme = segment-nz-nc *( "/" segment )
/// ```
fn path_noscheme(i: &[u8]) -> IResult<&[u8], Path> {
    let (rest, (nz, segments)) = pair(segment_nz_nc, path_abempty)(i)?;
    let segments = match segments {
        Path::AbEmpty(path) => path,
        _ => unreachable!(),
    };
    let (i, o) = split_input_to_str(i, nz.len() + segments.len());
    Ok((i, Path::NoScheme(o)))
}
/// ```ignore
/// path-rootless = segment-nz *( "/" segment )
/// ```
fn path_rootless(i: &[u8]) -> IResult<&[u8], Path> {
    let (rest, (nz, segments)) = pair(segment_nz, path_abempty)(i)?;
    let segments = match segments {
        Path::AbEmpty(path) => path,
        _ => unreachable!(),
    };
    let (i, o) = split_input_to_str(i, nz.len() + segments.len());
    Ok((rest, Path::Rootless(o)))
}
/// ```ignore
/// path-abempty  = *( "/" segment )
/// ```
fn path_abempty(i: &[u8]) -> IResult<&[u8], Path> {
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

/// ```ignore
/// path-empty    = 0<pchar>
/// ```
fn path_empty(i: &[u8]) -> IResult<&[u8], Path> {
    not(peek(pchar))(i)?;
    Ok((i, Path::Empty))
}
/// ```ignore
/// segment       = *pchar
/// ```
fn segment(i: &[u8]) -> IResult<&[u8], &str> {
    let (_, position) = fold_many0(pchar, 0, |mut pos: usize, _| {
        pos = fold_closure!(i, pos);
        pos
    })(i)?;
    Ok(split_input_to_str(i, position))
}
/// ```ignore
/// segment-nz    = 1*pchar
/// ```
fn segment_nz(i: &[u8]) -> IResult<&[u8], &str> {
    let (_, position) = fold_many1(pchar, 0, |mut pos: usize, _| {
        pos = fold_closure!(i, pos);
        pos
    })(i)?;
    Ok(split_input_to_str(i, position))
}
/// ```ignore
/// segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
/// non-zero-length segment without any colon ":"
/// ```
fn segment_nz_nc(i: &[u8]) -> IResult<&[u8], &str> {
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
/// ```ignore
/// pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
fn pchar(i: &[u8]) -> IResult<&[u8], char> {
    alt((unreserved, pct_encoded, sub_delims, one_of(":@")))(i)
}
/// ```ignore
/// query         = *( pchar / "/" / "?" )
/// ```
fn query(i: &[u8]) -> IResult<&[u8], Query> {
    let (_, position) = fold_many0(alt((pchar, one_of("/?"))), 0, |mut pos: usize, _| {
        pos = fold_closure!(i, pos);
        pos
    })(i)?;
    let (i, o) = split_input_to_str(i, position);
    Ok((i, Query(o)))
}
/// ```ignore
/// fragment      = *( pchar / "/" / "?" )
/// ```
fn fragment(i: &[u8]) -> IResult<&[u8], Fragment> {
    let (i, o) = match query(i)? {
        (i, Query(o)) => (i, o),
        _ => return Err(nom::Err::Error((i, ErrorKind::Many0))), //TODO: What error?
    };
    Ok((i, Fragment(o)))
}
/// ```ignore
/// percentage encoded u32
/// pct-encoded   = "%" HEXDIG HEXDIG
/// ```
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
/// ```ignore
/// unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
/// ```
fn unreserved(i: &[u8]) -> IResult<&[u8], char> {
    alt((alphanumeric, one_of("-._~")))(i)
}
/// ```ignore
/// reserved      = gen-delims / sub-delims
/// ```
fn reserved(i: &[u8]) -> IResult<&[u8], char> {
    alt((gen_delims, sub_delims))(i)
}
/// ```ignore
/// gen-delims    = ":" / "/" / "?" / "#" / "[" / "]" / "@"
/// ```
fn gen_delims(i: &[u8]) -> IResult<&[u8], char> {
    one_of(":/?#[]@")(i)
}
/// ```ignore
/// sub-delims    = "!" / "$" / "&" / "'" / "(" / ")"
///               / "*" / "+" / "," / ";" / "="
/// ```
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
fn port_test() {
    assert_eq!(port(b""), Ok((&b""[..], None)));
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
        ip_v4_address(b"255.255.255.255.255"),
        Ok((&b".255"[..], Host::V4("255.255.255.255")))
    );
    assert_eq!(
        ip_v4_address(b"255.255.255.255"),
        Ok((&b""[..], Host::V4("255.255.255.255")))
    );
    assert_eq!(
        ip_v4_address(b"0.0.0.0"),
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
        path_absolute(b"/abc/def//"),
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
        path_rootless(b"abc/def//"),
        Ok((&b""[..], Path::Rootless("abc/def//")))
    );
}
#[test]
fn path_abempty_test() {
    assert_eq!(
        path_abempty(b"/abc/def//"),
        Ok((&[][..], Path::AbEmpty("/abc/def//")))
    );
    assert_eq!(
        path_abempty(b"abc/def//"),
        Ok((&b"abc/def//"[..], Path::AbEmpty("")))
    );
}
#[test]
fn fragment_test() {
    unsafe {
        assert_eq!(
            fragment(pchar_no_pct),
            Ok((
                &[][..],
                Fragment(core::str::from_utf8_unchecked(&pchar_no_pct))
            ))
        )
    };
    assert_eq!(fragment(b"/?{"), Ok((&b"{"[..], Fragment("/?"))));
    assert_eq!(fragment(b"%30%41#"), Ok((&b"#"[..], Fragment("%30%41"))));
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
