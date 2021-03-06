use std::collections::HashMap;
use rustc_serialize::json::Json;

use conduit::{Handler, Request, Method};
use semver;

use cargo_registry::db::RequestTransaction;
use cargo_registry::version::{EncodableVersion, Version};

#[derive(RustcDecodable)]
struct VersionList { versions: Vec<EncodableVersion> }
#[derive(RustcDecodable)]
struct VersionResponse { version: EncodableVersion }

fn sv(s: &str) -> semver::Version {
    semver::Version::parse(s).unwrap()
}

#[test]
fn index() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app, Method::Get, "/api/v1/versions");
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionList = ::json(&mut response);
    assert_eq!(json.versions.len(), 0);

    let (v1, v2) = {
        ::mock_user(&mut req, ::user("foo"));
        let (c, _) = ::mock_crate(&mut req, ::krate("foo_vers_index"));
        let req: &mut Request = &mut req;
        let tx = req.tx().unwrap();
        let m = HashMap::new();
        let v1 = Version::insert(tx, c.id, &sv("2.0.0"), &m, &[]).unwrap();
        let v2 = Version::insert(tx, c.id, &sv("2.0.1"), &m, &[]).unwrap();
        (v1, v2)
    };
    req.with_query(&format!("ids[]={}&ids[]={}", v1.id, v2.id));
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionList = ::json(&mut response);
    assert_eq!(json.versions.len(), 2);
}

#[test]
fn show() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app, Method::Get, "/api/v1/versions");
    let v = {
        ::mock_user(&mut req, ::user("foo"));
        let (krate, _) = ::mock_crate(&mut req, ::krate("foo_vers_show"));
        let req: &mut Request = &mut req;
        let tx = req.tx().unwrap();
        Version::insert(tx, krate.id, &sv("2.0.0"), &HashMap::new(), &[]).unwrap()
    };
    req.with_path(&format!("/api/v1/versions/{}", v.id));
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionResponse = ::json(&mut response);
    assert_eq!(json.version.id, v.id);
}

#[test]
fn authors() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app, Method::Get, "/api/v1/crates/foo_authors/1.0.0/authors");
    ::mock_user(&mut req, ::user("foo"));
    ::mock_crate(&mut req, ::krate("foo_authors"));
    let mut response = ok_resp!(middle.call(&mut req));
    let mut s = String::new();
    response.body.read_to_string(&mut s).unwrap();
    let json = Json::from_str(&s).unwrap();
    let json = json.as_object().unwrap();
    assert!(json.contains_key(&"users".to_string()));
}
