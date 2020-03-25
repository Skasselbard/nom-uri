extern crate nom_uri;

#[test]
fn parser() {
    use nom_uri::Uri;
    let uri = Uri::parse("ftp://rms@example.com").unwrap();
    assert!(uri.has_host());

    let uri = Uri::parse("https://example.com/api/versions?page=2").unwrap();
    assert_eq!(uri.path(), "/api/versions");

    let uri = Uri::parse("https://example.com/foo/bar").unwrap();
    let mut path_segments = uri.path_segments();
    assert_eq!(path_segments.next(), Some("foo"));
    assert_eq!(path_segments.next(), Some("bar"));
    assert_eq!(path_segments.next(), None);
}

#[test]
fn formatter() {
    use nom_uri::Uri;
    let uri_str = "ftp://rms@example.com";
    let uri = Uri::parse(uri_str).unwrap();
    let buffer = &mut [b' '; 50][..];
    assert_eq!(uri_str, uri.as_str(buffer).unwrap());

    let uri_str = "ftp://rms@example.com/example/path";
    let uri = Uri::parse(uri_str).unwrap();
    assert_eq!(uri_str, uri.as_str(buffer).unwrap());

    let mut uri = Uri::parse("https://example.com/data.csv").unwrap();
    assert_eq!(uri.as_str(buffer).unwrap(), "https://example.com/data.csv");
    uri.set_fragment(Some("cell=4,1-6,2")).unwrap();
    let uri_str = uri.as_str(buffer).unwrap();
    assert_eq!(uri_str, "https://example.com/data.csv#cell=4,1-6,2");

    let mut uri = Uri::parse("https://example.com").unwrap();
    uri.set_path("/api/comments").unwrap();
    let buffer = &mut [b' '; 50][..];
    assert_eq!(
        uri.as_str(buffer).unwrap(),
        "https://example.com/api/comments"
    );
    assert_eq!(uri.path(), "/api/comments");

    let mut uri = Uri::parse("ssh://example.net:2048/").unwrap();
    uri.set_port(Some("4096")).unwrap();
    let buffer = &mut [b' '; 50][..];
    assert_eq!(uri.as_str(buffer).unwrap(), "ssh://example.net:4096/");
}
