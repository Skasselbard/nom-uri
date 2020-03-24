use super::*;
use core::fmt;

impl<'uri> fmt::Display for Uri<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}{}{}{}{}{}{}",
            self.scheme(),
            if self.authority.is_some() { "//" } else { "" },
            self.authority.unwrap_or(Authority {
                userinfo: None,
                host: Host::RegistryName(""),
                port: None
            }),
            self.path,
            if self.query.is_some() { "?" } else { "" },
            self.query.unwrap_or(Query("")),
            if self.fragment.is_some() { "#" } else { "" },
            self.fragment.unwrap_or(Fragment("")),
        )
    }
}
impl<'uri> fmt::Display for Authority<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.userinfo.unwrap_or(""),
            if self.userinfo.is_some() { "@" } else { "" },
            self.host,
            if self.port.is_some() { ":" } else { "" },
            self.port.unwrap_or("")
        )
    }
}
impl<'uri> fmt::Display for Host<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Host::RegistryName(host) | Host::V4(host) => write!(f, "{}", host),
            Host::V6(host) | Host::VFuture(host) => write!(f, "[{}]", host),
        }
    }
}
impl<'uri> fmt::Display for Path<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Path::AbEmpty(path)
                | Path::Absolute(path)
                | Path::NoScheme(path)
                | Path::Rootless(path) => {
                    path
                }
                Path::Empty => {
                    ""
                }
            }
        )
    }
}
impl<'uri> fmt::Display for Fragment<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<'uri> fmt::Display for Query<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
pub struct Buffer<'a> {
    buffer: &'a mut [u8],
    cursor: usize,
}
impl<'a> Buffer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }
    pub fn buffer(self) -> &'a mut [u8] {
        let (o, _) = self.buffer.split_at_mut(self.cursor);
        o
    }
}
impl<'a> fmt::Write for Buffer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let s = s.as_bytes();
        if self.buffer.len() - self.cursor < s.len() {
            Err(fmt::Error)
        } else {
            for i in 0..s.len() {
                self.buffer[self.cursor] = s[i];
                self.cursor += 1;
            }
            Ok(())
        }
    }
}
