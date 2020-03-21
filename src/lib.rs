mod parser;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Uri<'uri> {
    scheme: &'uri str,
    authority: Authority<'uri>,
    path: &'uri str,
    query: Option<&'uri str>,
    fragment: Option<&'uri str>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Authority<'uri> {
    userinfo: Option<&'uri str>,
    host: Host<'uri>,
    port: Option<u16>,
}
//TODO: can be part of uri struct
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UriPart<'uri> {
    Fragment(&'uri str),
    Query(&'uri str),
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
