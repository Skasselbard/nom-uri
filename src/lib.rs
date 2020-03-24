mod error;
mod formater;
///
/// - developed for no_std environments
/// - parsing is completely in memory
/// - no spaces allowed
/// - no implicit percent encoding of unallowed characters
/// - interface (and documentation) inspired by https://crates.io/crates/url
///    - but some things are different
///    - url is feature richer
///    - no default values (for ports)
///    - no scheme invariant checking (like absence of host for special schemes)
mod parser;

pub use error::Error;
use error::*;

#[derive(Debug, PartialEq)]
#[allow(unused)]
enum UriReference<'uri> {
    Uri(Uri<'uri>),
    Reference(Reference<'uri>),
}
#[derive(Debug, PartialEq)]
pub struct Uri<'uri> {
    scheme: &'uri str,
    authority: Option<Authority<'uri>>,
    path: Path<'uri>,
    query: Option<Query<'uri>>,
    fragment: Option<Fragment<'uri>>,
}
#[derive(Debug, PartialEq)]
struct Reference<'uri> {
    authority: Option<Authority<'uri>>,
    path: Path<'uri>,
    query: Option<Query<'uri>>,
    fragment: Option<Fragment<'uri>>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Authority<'uri> {
    userinfo: Option<&'uri str>,
    host: Host<'uri>,
    port: Option<&'uri str>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Host<'uri> {
    RegistryName(&'uri str),
    V4(&'uri str),
    V6(&'uri str),
    VFuture(&'uri str),
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum Path<'uri> {
    AbEmpty(&'uri str),
    Absolute(&'uri str),
    NoScheme(&'uri str),
    Rootless(&'uri str),
    Empty,
}
#[derive(Debug, PartialEq, Clone, Copy)]
struct Fragment<'uri>(&'uri str);
#[derive(Debug, PartialEq, Clone, Copy)]
struct Query<'uri>(&'uri str);

impl<'uri> Uri<'uri> {
    /// Parse an URI from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.net")?;
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn parse(input: &'uri str) -> Result<Self, Error> {
        Self::parse_bytes(input.as_bytes())
    }
    /// Parse an URI from a byte slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse_bytes(b"https://example.net")?;
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn parse_bytes(input: &'uri [u8]) -> Result<Self, Error> {
        match parser::uri::<ParserError>(input) {
            Ok((_, o)) => Ok(o),
            Err(e) => Err(nom_error_to_error(e)),
        }
    }
    /// Return the serialization of this URI.
    ///
    /// Since a uri does not own the parsed bytes mutably,
    /// we need a buffer which is used for the output.
    /// The returned &str is a subslice of the input buffer.
    ///
    /// All characters in an uri are ascii characters (unicode characters
    /// have to be percent encoded: "%00" - "%FF").
    /// Therefore the length of the return string should match the byte count
    /// used in the buffer: ``return.len() == return.as_bytes().len()``
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri_str = "ftp://rms@example.com";
    /// let uri = Uri::parse(uri_str).unwrap();
    /// let buffer = &mut [b'x'; 30][..];
    /// assert_eq!(uri_str, uri.as_str(buffer).unwrap());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn as_str<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut str, Error> {
        use core::fmt::Write;
        let mut buffer = formater::Buffer::new(buffer);
        if write!(buffer, "{}", self).is_err() {
            return Err(Error::BufferToSmall);
        }
        unsafe { Ok(core::str::from_utf8_unchecked_mut(buffer.buffer())) }
    }

    /// TODO: doc
    /// absolute uri
    /// omit the fragment
    #[inline]
    pub fn base(&self) -> Option<&str> {
        unimplemented!()
    }

    /// Return the scheme of this URI, as an ASCII string without the ':' delimiter.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("file:///tmp/foo")?;
    /// assert_eq!(uri.scheme(), "file");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn scheme(&self) -> &str {
        self.scheme
    }

    /// Return whether the URI has an 'authority',
    /// which can contain a username, password, host, and port number.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("ftp://rms@example.com")?;
    /// assert!(uri.has_authority());
    ///
    /// let uri = Uri::parse("unix:/run/foo.socket")?;
    /// assert!(!uri.has_authority());
    ///
    /// let uri = Uri::parse("data:text/plain,Stuff")?;
    /// assert!(!uri.has_authority());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn has_authority(&self) -> bool {
        self.authority.is_some()
    }

    /// Return the userinfo for this URI.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("ftp://rms@example.com")?;
    /// assert_eq!(uri.userinfo(), Some("rms"));
    ///
    /// let uri = Uri::parse("https://example.com")?;
    /// assert_eq!(uri.userinfo(), None);
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn userinfo(&self) -> Option<&str> {
        match self.authority {
            Some(auth) => auth.userinfo,
            None => None,
        }
    }

    /// # Examples
    /// Returns wether the uri has a host. The host is required in the authority part,
    /// so if an uri has no host, it also has no authority.
    ///
    /// ```
    /// use nom_uri::Uri;
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("ftp://rms@example.com")?;
    /// assert!(uri.has_host());
    ///
    /// let uri = Uri::parse("unix:/run/foo.socket")?;
    /// assert!(!uri.has_host());
    ///
    /// let uri = Uri::parse("data:text/plain,Stuff")?;
    /// assert!(!uri.has_host());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn has_host(&self) -> bool {
        self.has_authority()
    }

    /// Return the string representation of the host (domain or IP address) for this URI, if any.
    ///
    /// Non-ASCII domains are punycode-encoded per IDNA.
    /// IPv6 addresses are given between `[` and `]` brackets.
    ///
    /// See also the `host` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://127.0.0.1/index.html")?;
    /// assert_eq!(uri.host_str(), Some("127.0.0.1"));
    ///
    /// let uri = Uri::parse("ftp://rms@example.com")?;
    /// assert_eq!(uri.host_str(), Some("example.com"));
    ///
    /// let uri = Uri::parse("unix:/run/foo.socket")?;
    /// assert_eq!(uri.host_str(), None);
    ///
    /// let uri = Uri::parse("data:text/plain,Stuff")?;
    /// assert_eq!(uri.host_str(), None);
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn host_str(&self) -> Option<&str> {
        match self.authority {
            Some(auth) => match auth.host {
                Host::RegistryName(name) => Some(name),
                Host::V4(addr) => Some(addr),
                Host::V6(addr) => Some(addr),
                Host::VFuture(_addr) => unimplemented!(),
            },
            None => None,
        }
    }

    /// Return the parsed representation of the host for this URI.
    ///
    /// See also the `host_str` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://127.0.0.1/index.html")?;
    /// assert!(uri.host().is_some());
    ///
    /// let uri = Uri::parse("ftp://rms@example.com")?;
    /// assert!(uri.host().is_some());
    ///
    /// let uri = Uri::parse("unix:/run/foo.socket")?;
    /// assert!(uri.host().is_none());
    ///
    /// let uri = Uri::parse("data:text/plain,Stuff")?;
    /// assert!(uri.host().is_none());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn host(&self) -> Option<Host> {
        match self.authority {
            Some(auth) => Some(auth.host),
            None => None,
        }
    }

    /// If this URI has a host and it is a domain name (not an IP address), return it.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://127.0.0.1/")?;
    /// assert_eq!(uri.domain(), None);
    ///
    /// let uri = Uri::parse("mailto:rms@example.net")?;
    /// assert_eq!(uri.domain(), None);
    ///
    /// let uri = Uri::parse("https://example.com/")?;
    /// assert_eq!(uri.domain(), Some("example.com"));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn domain(&self) -> Option<&str> {
        match self.host() {
            Some(Host::RegistryName(name)) => Some(name),
            _ => None,
        }
    }

    /// Return the port number for this URI, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.com")?;
    /// assert_eq!(uri.port(), None);
    ///
    /// let uri = Uri::parse("https://example.com:443/")?;
    /// assert_eq!(uri.port(), Some(443));
    ///
    /// let uri = Uri::parse("ssh://example.com:22")?;
    /// assert_eq!(uri.port(), Some(22));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn port(&self) -> Option<u16> {
        match self.authority {
            Some(auth) => match auth.port {
                // parsing checked the conversion already
                Some(port) => Some(u16::from_str_radix(port, 10).unwrap()),
                None => None,
            },
            None => None,
        }
    }
    /// Return the path for this URI, as a percent-encoded ASCII string.
    /// For cannot-be-a-base URIs, this is an arbitrary string that doesn’t start with '/'.
    /// For other URIs, this starts with a '/' slash
    /// and continues with slash-separated path segments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.com/api/versions?page=2")?;
    /// assert_eq!(uri.path(), "/api/versions");
    ///
    /// let uri = Uri::parse("https://example.com")?;
    /// assert_eq!(uri.path(), "");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn path(&self) -> &str {
        match self.path {
            Path::AbEmpty(p) => p,
            Path::Absolute(p) => p,
            Path::NoScheme(p) => p,
            Path::Rootless(p) => p,
            Path::Empty => "",
        }
    }

    /// Unless this URI is cannot-be-a-base,
    /// return an iterator of '/' slash-separated path segments,
    /// each as a percent-encoded ASCII string.
    ///
    /// Return `None` for cannot-be-a-base URIs.
    ///
    /// When `Some` is returned, the iterator always contains at least one string
    /// (which may be empty).
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    /// # use std::error::Error;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.com/foo/bar")?;
    /// let mut path_segments = uri.path_segments().ok_or_else(|| "cannot be base")?;
    /// assert_eq!(path_segments.next(), Some("foo"));
    /// assert_eq!(path_segments.next(), Some("bar"));
    /// assert_eq!(path_segments.next(), None);
    ///
    /// let uri = Uri::parse("https://example.com")?;
    /// let mut path_segments = uri.path_segments().ok_or_else(|| "cannot be base")?;
    /// assert_eq!(path_segments.next(), Some(""));
    /// assert_eq!(path_segments.next(), None);
    ///
    /// let uri = Uri::parse("data:text/plain,HelloWorld")?;
    /// assert!(uri.path_segments().is_none());
    ///
    /// let uri = Uri::parse("https://example.com/countries/việt nam")?;
    /// let mut path_segments = uri.path_segments().ok_or_else(|| "cannot be base")?;
    /// assert_eq!(path_segments.next(), Some("countries"));
    /// assert_eq!(path_segments.next(), Some("vi%E1%BB%87t%20nam"));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn path_segments(&self) -> Option<core::str::Split<char>> {
        // FIXME:
        if self.path() != "" {
            Some(self.path()[1..].split('/'))
        } else {
            None
        }
    }

    /// Return this URI’s query string, if any, as a percent-encoded ASCII string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.com/products?page=2")?;
    /// let query = uri.query();
    /// assert_eq!(query, Some("page=2"));
    ///
    /// let uri = Uri::parse("https://example.com/products")?;
    /// let query = uri.query();
    /// assert!(query.is_none());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn query(&self) -> Option<&str> {
        match self.query {
            Some(Query(q)) => Some(q),
            None => None,
        }
    }

    /// Parse the URI’s query string, if any, as `application/x-www-form-uriencoded`
    /// and return an iterator of (key, value) pairs.
    ///
    /// # Examples
    ///
    /// ```compile_fail
    /// use std::borrow::Cow;
    ///
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.com/products?page=2&sort=desc")?;
    /// let mut pairs = uri.query_pairs();
    ///
    /// assert_eq!(pairs.count(), 2);
    ///
    /// assert_eq!(pairs.next(), Some((Cow::Borrowed("page"), Cow::Borrowed("2"))));
    /// assert_eq!(pairs.next(), Some((Cow::Borrowed("sort"), Cow::Borrowed("desc"))));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    ///
    #[inline]
    pub fn query_pairs(&self) -> &[(&str, &str)] {
        // FIXME:
        unimplemented!()
    }

    /// Return this URI’s fragment identifier, if any.
    ///
    /// A fragment is the part of the URI after the `#` symbol.
    /// The fragment is optional and, if present, contains a fragment identifier
    /// that identifies a secondary resource, such as a section heading
    /// of a document.
    ///
    /// In HTML, the fragment identifier is usually the id attribute of a an element
    /// that is scrolled to on load. Browsers typically will not send the fragment portion
    /// of a URI to the server.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let uri = Uri::parse("https://example.com/data.csv#row=4")?;
    ///
    /// assert_eq!(uri.fragment(), Some("row=4"));
    ///
    /// let uri = Uri::parse("https://example.com/data.csv#cell=4,1-6,2")?;
    ///
    /// assert_eq!(uri.fragment(), Some("cell=4,1-6,2"));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn fragment(&self) -> Option<&str> {
        match self.fragment {
            Some(Fragment(f)) => Some(f),
            None => None,
        }
    }

    /// Change this URI’s fragment identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let buffer = &mut [b' '; 50][..];
    /// let mut uri = Uri::parse("https://example.com/data.csv")?;
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/data.csv");
    /// uri.set_fragment(Some("cell=4,1-6,2"));
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/data.csv#cell=4,1-6,2");
    /// assert_eq!(uri.fragment(), Some("cell=4,1-6,2"));
    ///
    /// uri.set_fragment(None);
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/data.csv");
    /// assert!(uri.fragment().is_none());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_fragment<'a: 'uri>(&mut self, fragment: Option<&'a str>) -> Result<(), Error> {
        self.fragment = match fragment {
            Some(fragment) => match parser::fragment::<ParserError>(fragment.as_bytes()) {
                Ok((_, f)) => Some(f),
                Err(e) => return Err(nom_error_to_error(e)),
            },
            None => None,
        };
        Ok(())
    }

    /// Change this URI’s query string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let buffer = &mut [b' '; 50][..];
    /// let mut uri = Uri::parse("https://example.com/products")?;
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/products");
    ///
    /// uri.set_query(Some("page=2"));
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/products?page=2");
    /// assert_eq!(uri.query(), Some("page=2"));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_query<'a: 'uri>(&mut self, query: Option<&'a str>) -> Result<(), Error> {
        self.query = match query {
            Some(query) => match parser::query::<ParserError>(query.as_bytes()) {
                Ok((_, q)) => Some(q),
                Err(e) => return Err(nom_error_to_error(e)),
            },
            None => None,
        };
        Ok(())
    }

    /// Change this URI’s path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let mut uri = Uri::parse("https://example.com")?;
    /// uri.set_path("/api/comments");
    /// let buffer = &mut [b' '; 50][..];
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/api/comments");
    /// assert_eq!(uri.path(), "/api/comments");
    ///
    /// let mut uri = Uri::parse("https://example.com/api")?;
    /// uri.set_path("/data/report.csv");
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/data/report.csv");
    /// assert_eq!(uri.path(), "/data/report.csv");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_path<'a: 'uri>(&mut self, path: &'a str) -> Result<(), Error> {
        // TODO:check that the path type is valid for the rest of the uri
        match parser::path::<ParserError>(path.as_bytes()) {
            Ok((_, p)) => self.path = p,
            Err(e) => return Err(nom_error_to_error(e)),
        };
        Ok(())
    }

    /// Change this URI’s port number.
    ///
    /// # Examples
    ///
    /// ```
    /// use nom_uri::Uri;
    /// # use std::error::Error;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let mut uri = Uri::parse("ssh://example.net:2048/")?;
    ///
    /// uri.set_port(Some("4096"))?;
    /// let buffer = &mut [b' '; 50][..];
    /// assert_eq!(uri.as_str(buffer).unwrap(), "ssh://example.net:4096/");
    ///
    /// uri.set_port(None);
    /// assert_eq!(uri.as_str(buffer).unwrap(), "ssh://example.net/");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_port<'a: 'uri>(&mut self, port: Option<&'a str>) -> Result<(), Error> {
        match self.authority.as_mut() {
            Some(auth) => match port {
                Some(port) => match parser::port::<ParserError>(port.as_bytes()) {
                    Ok((_, p)) => {
                        // dbg!(&p);
                        // dbg!(&auth);
                        auth.port = p;
                        // dbg!(&auth);
                    }
                    Err(e) => return Err(nom_error_to_error(e)),
                },
                None => auth.port = None,
            },
            None => return Err(Error::NoAuthority),
        };
        Ok(())
    }

    /// Change this URI’s host.
    ///
    /// Removing the host (calling this with `None`)
    /// will also remove any username, password, and port number.
    ///
    /// # Examples
    ///
    /// Change host:
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let mut uri = Uri::parse("https://example.net")?;
    /// let result = uri.set_host(Some("rust-lang.org"));
    /// assert!(result.is_ok());
    /// let buffer = &mut [b' '; 50][..];
    /// assert_eq!(uri.as_str(buffer).unwrap(), "https://rust-lang.org");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    ///
    /// Remove host:
    ///
    /// ```
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let mut uri = Uri::parse("foo://example.net")?;
    /// let result = uri.set_host(None);
    /// assert!(result.is_ok());
    /// let buffer = &mut [b' '; 50][..];
    /// assert_eq!(uri.as_str(buffer).unwrap(), "foo:");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_host<'a: 'uri>(&mut self, host: Option<&'a str>) -> Result<(), Error> {
        match host {
            None => self.authority = None,
            Some(host) => match self.authority.as_mut() {
                Some(auth) => {
                    auth.host = match parser::host::<ParserError>(host.as_bytes()) {
                        Ok((_, host)) => host,
                        Err(e) => return Err(nom_error_to_error(e)),
                    }
                }
                None => return Err(Error::NoAuthority),
            },
        };
        Ok(())
    }
    /// Change this URI’s userinfo.
    ///
    /// # Examples
    /// Setup userinfo to user1
    ///
    /// ```
    /// use uri::{Uri, ()};
    ///
    /// # fn run() -> Result<(), nom_uri::Error<'static>> {
    /// let mut uri = Uri::parse("ftp://example.com/")?;
    /// let result = uri.set_userinfo("user1");
    /// let buffer = &mut [b' '; 50][..];
    /// assert!(result.is_ok());
    /// assert_eq!(uri.userinfo(), "user1");
    /// assert_eq!(uri.as_str(), "ftp://user1@example.com/");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_userinfo(&mut self, _username: &str) -> Result<(), Error> {
        unimplemented!()
    }

    /// Change this URI’s scheme.
    pub fn set_scheme<'a: 'uri>(&mut self, scheme: &'a str) -> Result<(), Error> {
        self.scheme = match parser::scheme::<ParserError>(scheme.as_bytes()) {
            Ok((_, scheme)) => scheme,
            Err(e) => return Err(nom_error_to_error(e)),
        };
        Ok(())
    }
}
impl<'uri> Authority<'uri> {
    pub fn len(&self) -> usize {
        self.userinfo.unwrap_or("").len() + self.host.len() + self.port.unwrap_or("").len()
    }
}
impl<'uri> Host<'uri> {
    pub fn len(&self) -> usize {
        match self {
            Host::RegistryName(s) | Host::VFuture(s) | Host::V4(s) | Host::V6(s) => s.len(),
        }
    }
}
impl<'uri> Path<'uri> {
    pub fn len(&self) -> usize {
        match self {
            Path::AbEmpty(s) | Path::Absolute(s) | Path::NoScheme(s) | Path::Rootless(s) => s.len(),
            Path::Empty => 0,
        }
    }
}
impl<'uri> Query<'uri> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl<'uri> Fragment<'uri> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
