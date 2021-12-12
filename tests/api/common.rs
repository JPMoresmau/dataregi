use dataregi::model::Member;
use rocket::serde::DeserializeOwned;
use rocket::local::blocking::{Client,LocalRequest};
use std::env;

use dataregi::{docs::DocumentUpload, base::UserContext};
use std::fs;
use rocket::http::{ContentType, Status, Header, Cookie};
use uuid::Uuid;

pub fn setup() -> Client{
    env::set_var("ROCKET_PROFILE","test");

    let rocket = dataregi::rocket();
    Client::tracked(rocket).unwrap()
}

pub fn with_test_login(req: LocalRequest, user_idx: u8) -> LocalRequest {
    let ctx=UserContext::new(Uuid::parse_str(&format!("b9518d55-3256-4b96-81d0-65b1d7c4fb3{}",user_idx)).unwrap(),user_idx==1);
    req.private_cookie(Cookie::new("user", serde_json::to_string(&ctx).unwrap()))
}

pub fn with_org_login<'a>(req: LocalRequest<'a>, user_idx: u8, members: &[Member]) -> LocalRequest<'a> {
    let uuid=Uuid::parse_str(&format!("b9518d55-3256-4b96-81d0-65b1d7c4fb3{}",user_idx)).unwrap();
    let ctx=UserContext::new_in_org(uuid,
            members, 
    user_idx==1);
    req.private_cookie(Cookie::new("user", serde_json::to_string(&ctx).unwrap()))
}

pub fn do_upload(client: &Client, path: &str, user_idx: u8) -> DocumentUpload {
    do_upload_org(client, path, user_idx, None)
}

pub fn do_upload_org(client: &Client, path: &str, user_idx: u8, member: Option<Member>) -> DocumentUpload {

    let file = fs::read(path).unwrap();
    let mut cnt=vec![];

    cnt.extend("-----------------------------3511489321811197009899980000\r\n".as_bytes());    
    cnt.extend(format!("Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n",path).as_bytes());
    //cnt.extend("Content-Transfer-Encoding: binary\r\n".as_bytes());
    cnt.extend("Content-Type: application/vnd.oasis.opendocument.spreadsheet\r\n\r\n".as_bytes());
    cnt.extend(&file);
    cnt.extend("\r\n-----------------------------3511489321811197009899980000--\r\n".as_bytes()); 

    let url = match &member {
        None=>String::from("/api/docs"),
        Some(m)=>format!("/api/docs?org={}",m.org_id),
    };

    let req = with_org_login(client.post(url), user_idx,&member.into_iter().collect::<Vec<Member>>())
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
    upload_user(client,path,1)
}

pub fn upload_user(client: &Client, path: &str, user_idx: u8) -> Uuid {
    match do_upload(client,path, user_idx) {
        DocumentUpload::Ok{id} => id,
        du => panic!("unexpected upload result:{:?}",du),
    }
}


pub fn upload_org(client: &Client, path: &str, member: Member) -> Uuid {
    match do_upload_org(client,path,1, Some(member)) {
        DocumentUpload::Ok{id} => id,
        du => panic!("unexpected upload result:{:?}",du),
    }
}

pub fn delete(client: &Client, uuids: &[Uuid]) {
    delete_user(client, uuids, 1)
}

pub fn delete_user(client: &Client, uuids: &[Uuid], user_idx: u8) {
    for uuid in uuids.iter(){
        let response= with_test_login(client.delete(format!("/api/docs/{}",uuid)), user_idx).dispatch();
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
