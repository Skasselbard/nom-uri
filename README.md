# nom-uri
``no_std`` uri parser build with [nom](https://crates.io/crates/nom).

Based on the grammar from http://www.faqs.org/rfcs/rfc3986.html appendix A.

All parsing is done completely in memory.

## Example 
```rust
extern crate nom_uri;

use nom_uri::Uri;
use nom_uri::Host;

fn main() {
    // Parsing
    let uri = Uri::parse("https://127.0.0.1.com/api/versions?page=2").unwrap();
    assert_eq!(uri.host(), Some(Host::V4("127.0.0.1")));
    let uri = Uri::parse("https://example.com/foo/bar").unwrap();
    let mut path_segments = uri.path_segments();
    assert_eq!(path_segments.next(), Some("foo"));
    assert_eq!(path_segments.next(), Some("bar"));
    assert_eq!(path_segments.next(), None);

    // Serializing
    let mut uri = Uri::parse("https://example.com/data.csv").unwrap();
    let buffer = &mut [b' '; 50][..];
    assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/data.csv");
    uri.set_fragment(Some("cell=4,1-6,2")).unwrap();
    let uri_str = uri.as_str(buffer).unwrap();
    assert_eq!(uri_str, "https://example.com/data.csv#cell=4,1-6,2");
}
```
## Relation to [url](https://crates.io/crates/url) crate
This crate was build for ``no_std`` environments and to be used without allocator. A nice side effect is that parsing can be done completely in memory. The returned ``Uri`` object stores the parsed components as slices of the original input.

[URL](https://crates.io/crates/url) is better suited for normal rust programs with access to the standard library. This crate is inspired by the URL crate but differs in some details.
For example this crate does not parse special characters (including spaces!). They have to be percent encoded **before** parsing ([space] => "%20"). The URL crate does this automatically.
In general the URL crate has more advanced features, especially for everyday URL handling.