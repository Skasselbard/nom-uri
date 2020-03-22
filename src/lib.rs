mod parser;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UriReference<'uri> {
    Uri(Uri<'uri>),
    Reference(Reference<'uri>),
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Uri<'uri> {
    scheme: &'uri str,
    authority: Authority<'uri>,
    path: Path<'uri>,
    query: Option<Query<'uri>>,
    fragment: Option<Fragment<'uri>>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reference<'uri> {
    authority: Authority<'uri>,
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
pub enum Path<'uri> {
    AbEmpty(&'uri str),
    Absolute(&'uri str),
    NoScheme(&'uri str),
    Rootless(&'uri str),
    Empty,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Fragment<'uri>(&'uri str);
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Query<'uri>(&'uri str);

impl<'uri> Uri<'uri> {
    pub fn to_absoulte(&self) -> &str {
        ""
    }
}
