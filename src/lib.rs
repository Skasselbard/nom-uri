mod parser;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Uri<'uri> {
    Scheme(&'uri str),
    Authority(Authority<'uri>),
    Path(&'uri str),
    Query(&'uri str),
    Fragment(&'uri str),
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Authority<'uri> {
    Userinfo(&'uri str),
    Host(&'uri str),
    Port(u16),
}
