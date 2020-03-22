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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UriReference<'uri> {
    Uri(Uri<'uri>),
    Reference(Reference<'uri>),
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Uri<'uri> {
    scheme: &'uri str,
    authority: Option<Authority<'uri>>,
    path: Path<'uri>,
    query: Option<Query<'uri>>,
    fragment: Option<Fragment<'uri>>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reference<'uri> {
    authority: Option<Authority<'uri>>,
    path: Path<'uri>,
    query: Option<Query<'uri>>,
    fragment: Option<Fragment<'uri>>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Authority<'uri> {
    userinfo: Option<&'uri str>,
    host: Host<'uri>,
    port: Option<u16>,
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
    /// # fn run() -> Result<(), ()> {
    /// let uri = Uri::parse("https://example.net")?;
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn parse(input: &'uri str) -> Result<Self, ()> {
        Self::parse_bytes(input.as_bytes())
    }
    /// Parse an URI from a byte slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), ()> {
    /// let uri = Uri::parse_bytes(b"https://example.net")?;
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn parse_bytes(input: &'uri [u8]) -> Result<Self, ()> {
        match parser::uri(input) {
            Ok((_, o)) => Ok(o),
            Err(_) => Err(()),
        }
    }
    /// Return the serialization of this URI.
    ///
    /// This is fast since that serialization is already stored in the `Uri` struct.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), ()> {
    /// let uri_str = "https://example.net/";
    /// let uri = Uri::parse(uri_str)?;
    /// assert_eq!(uri.as_str(), uri_str);
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        unimplemented!()
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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

    /// Return the username for this URI (typically the empty string)
    /// as a percent-encoded ASCII string.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use nom_uri::Uri;
    ///
    /// # fn run() -> Result<(), ()> {
    /// let uri = Uri::parse("ftp://rms@example.com")?;
    /// assert_eq!(uri.username(), "rms");
    ///
    /// let uri = Uri::parse("ftp://:secret123@example.com")?;
    /// assert_eq!(uri.username(), "");
    ///
    /// let uri = Uri::parse("https://example.com")?;
    /// assert_eq!(uri.username(), "");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn username(&self) -> &str {
        unimplemented!()
    }

    /// # Examples
    /// Returns wether the uri has a host. The host is required in the authority part,
    /// so if an uri has no host, it also has no authority.
    ///
    /// ```
    /// use nom_uri::Uri;
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
            Some(auth) => auth.port,
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
    /// fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
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
    /// # fn run() -> Result<(), ()> {
    /// let mut uri = Uri::parse("https://example.com/data.csv")?;
    /// assert_eq!(uri.as_str(), "https://example.com/data.csv");
    /// uri.set_fragment(Some("cell=4,1-6,2"));
    /// assert_eq!(uri.as_str(), "https://example.com/data.csv#cell=4,1-6,2");
    /// assert_eq!(uri.fragment(), Some("cell=4,1-6,2"));
    ///
    /// uri.set_fragment(None);
    /// assert_eq!(uri.as_str(), "https://example.com/data.csv");
    /// assert!(uri.fragment().is_none());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_fragment<'a: 'uri>(&mut self, fragment: Option<&'a str>) -> Result<(), ()> {
        self.fragment = match fragment {
            Some(fragment) => match parser::fragment(fragment.as_bytes()) {
                Ok((_, f)) => Some(f),
                Err(_) => return Err(()),
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
    /// # fn run() -> Result<(), ()> {
    /// let mut uri = Uri::parse("https://example.com/products")?;
    /// assert_eq!(uri.as_str(), "https://example.com/products");
    ///
    /// uri.set_query(Some("page=2"));
    /// assert_eq!(uri.as_str(), "https://example.com/products?page=2");
    /// assert_eq!(uri.query(), Some("page=2"));
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_query<'a: 'uri>(&mut self, query: Option<&'a str>) -> Result<(), ()> {
        self.query = match query {
            Some(query) => match parser::query(query.as_bytes()) {
                Ok((_, q)) => Some(q),
                Err(_) => return Err(()),
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
    /// # fn run() -> Result<(), ()> {
    /// let mut uri = Uri::parse("https://example.com")?;
    /// uri.set_path("api/comments");
    /// assert_eq!(uri.as_str(), "https://example.com/api/comments");
    /// assert_eq!(uri.path(), "/api/comments");
    ///
    /// let mut uri = Uri::parse("https://example.com/api")?;
    /// uri.set_path("data/report.csv");
    /// assert_eq!(uri.as_str(), "https://example.com/data/report.csv");
    /// assert_eq!(uri.path(), "/data/report.csv");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_path<'a: 'uri>(&mut self, path: &'a str) -> Result<(), ()> {
        // TODO:check that the path type is valid for the rest of the uri
        self.path = match parser::path(path.as_bytes()) {
            Ok((_, p)) => p,
            Err(_) => return Err(()),
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
    /// # fn run() -> Result<(), Box<Error>> {
    /// let mut uri = Uri::parse("ssh://example.net:2048/")?;
    ///
    /// uri.set_port(Some(4096)).map_err(|_| "cannot be base")?;
    /// assert_eq!(uri.as_str(), "ssh://example.net:4096/");
    ///
    /// uri.set_port(None).map_err(|_| "cannot be base")?;
    /// assert_eq!(uri.as_str(), "ssh://example.net/");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_port(&mut self, port: Option<u16>) -> Result<(), ()> {
        match self.authority {
            Some(mut auth) => auth.port = port,
            None => return Err(()),
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
    /// # fn run() -> Result<(), ()> {
    /// let mut uri = Uri::parse("https://example.net")?;
    /// let result = uri.set_host(Some("rust-lang.org"));
    /// assert!(result.is_ok());
    /// assert_eq!(uri.as_str(), "https://rust-lang.org/");
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
    /// # fn run() -> Result<(), ()> {
    /// let mut uri = Uri::parse("foo://example.net")?;
    /// let result = uri.set_host(None);
    /// assert!(result.is_ok());
    /// assert_eq!(uri.as_str(), "foo:/");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_host(&mut self, host: &str) -> Result<(), ()> {
        match self.authority {
            Some(mut auth) => {
                auth.host = match parser::host(host.as_bytes()) {
                    Ok((_, host)) => host,
                    Err(_) => return Err(()),
                }
            }
            None => return Err(()),
        };
        Ok(())
    }
    /// Change this URI’s username.
    ///
    /// # Examples
    /// Setup username to user1
    ///
    /// ```should_panic
    /// use uri::{Uri, ()};
    ///
    /// # fn run() -> Result<(), ()> {
    /// let mut uri = Uri::parse("ftp://:secre1@example.com/")?;
    /// let result = uri.set_username("user1");
    /// assert!(result.is_ok());
    /// assert_eq!(uri.username(), "user1");
    /// assert_eq!(uri.as_str(), "ftp://user1:secre1@example.com/");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    pub fn set_username(&mut self, _username: &str) -> Result<(), ()> {
        unimplemented!()
    }

    /// Change this URI’s scheme.
    pub fn set_scheme<'a: 'uri>(&mut self, scheme: &'a str) -> Result<(), ()> {
        self.scheme = match parser::scheme(scheme.as_bytes()) {
            Ok((_, scheme)) => scheme,
            Err(_) => return Err(()),
        };
        Ok(())
    }
}
