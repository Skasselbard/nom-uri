extern crate nom_uri;

#[test]
fn parser() {
    use nom_uri::Uri;
    let uri = Uri::parse("ftp://rms@example.com").unwrap();
    assert!(uri.has_host());
    let uri = Uri::parse("https://example.com/api/versions?page=2").unwrap();
    assert_eq!(uri.path(), "/api/versions");
}
