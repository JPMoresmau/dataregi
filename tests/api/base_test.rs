use crate::common::{setup,with_test_login};
use rocket::http::{ContentType, Status};


#[test]
fn index_logged_out_returns_ok(){
    let client = setup();
    let req = client.get("/");
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    let cnt=response.into_string().unwrap();
    assert!(cnt.contains("<input id=\"email\" type=\"text\""));
    assert!(!cnt.contains("<input type=\"file\" id=\"uploadInput\""));
}


#[test]
fn index_logged_in_returns_ok(){
    let client = setup();
    let req = with_test_login(client.get("/"), 1);
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    let cnt=response.into_string().unwrap();
    assert!(cnt.contains("<input type=\"file\" id=\"uploadInput\""));
    assert!(!cnt.contains("<input id=\"email\" type=\"text\""));
}

#[test]
fn api_unauthorized(){
    let client = setup();
    let req = client.get("/api/orgs");
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Unauthorized);
}


#[test]
fn site_unauthorized(){
    let client = setup();
    let req = client.get("/profile");
    let response = req.dispatch();
    assert_eq!(response.status(),Status::SeeOther);
    
}