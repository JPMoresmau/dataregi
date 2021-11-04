use dataregi;
use rocket::serde::DeserializeOwned;
use rocket::local::blocking::{Client,LocalRequest};
use std::env;

use dataregi::{docs::DocumentUpload};
use std::fs;
use rocket::http::{ContentType, Status, Header, Cookie};
use uuid::Uuid;

pub fn setup() -> Client{
    env::set_var("ROCKET_PROFILE","test");

    let rocket = dataregi::rocket();
    let client = Client::tracked(rocket).unwrap();

    client
}

pub fn with_test_login(req: LocalRequest, user_idx: u8) -> LocalRequest {
    req.private_cookie(Cookie::new("id", format!("b9518d55-3256-4b96-81d0-65b1d7c4fb3{}",user_idx)))
}

pub fn do_upload(client: &Client, path: &str) -> DocumentUpload {
    let file = fs::read(path).unwrap();
    let mut cnt=vec![];

    cnt.extend("-----------------------------3511489321811197009899980000\r\n".as_bytes());    
    cnt.extend(format!("Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n",path).as_bytes());
    //cnt.extend("Content-Transfer-Encoding: binary\r\n".as_bytes());
    cnt.extend("Content-Type: application/vnd.oasis.opendocument.spreadsheet\r\n\r\n".as_bytes());
    cnt.extend(&file);
    cnt.extend("\r\n-----------------------------3511489321811197009899980000--\r\n".as_bytes()); 

    let req = with_test_login(client.post("/api/docs"), 1)
        .header(ContentType::with_params("multipart", "form-data", ("boundary", "---------------------------3511489321811197009899980000")))
        .header(Header::new("Content-Length",format!("{}",cnt.len())))
        .body(cnt);
    
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    // get id
    let mut uuids:Vec<DocumentUpload> = response.into_json().unwrap(); 

    assert_eq!(uuids.len(),1);
    uuids.pop().unwrap()
}

pub fn upload(client: &Client, path: &str) -> Uuid {
    match do_upload(client,path) {
        DocumentUpload::Ok{id} => id,
        du => panic!("unexpected upload result:{:?}",du),
    }
}

pub fn delete(client: &Client, uuids: &[Uuid]) {
    for uuid in uuids.iter(){
        let response= with_test_login(client.delete(format!("/api/docs/{}",uuid)), 1).dispatch();
        assert_eq!(response.status(),Status::NoContent);
    }
}

pub fn json_ok_response<T>(req: LocalRequest) -> T
where T: Send + DeserializeOwned + 'static {
    let response= req.dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    response.into_json().unwrap()
}
