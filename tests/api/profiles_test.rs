use dataregi::{model::{User}};
use crate::common::{setup,with_test_login,json_ok_response};

#[test]
fn get_profile() {
    let client= setup();
    let u:User=json_ok_response(with_test_login(client.get("/api/profiles"), 1));
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",&u.id.to_string());
    assert_eq!("test1@dataregi.com",&u.email);
    assert_eq!("Test User 1",&u.name);
    assert!(u.site_admin);
}