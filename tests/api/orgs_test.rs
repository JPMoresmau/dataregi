use dataregi::{model::{Organization, Member}};
use crate::common::{setup,with_test_login,do_upload,upload,delete, json_ok_response};
use rocket::http::{ContentType, Status};
use serial_test::serial;

#[test]
#[serial]
fn crud() {
    let client= setup();

    let lt:i64=json_ok_response(with_test_login(client.get("/api/orgs/count"), 1));
    assert_eq!(0,lt);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs/all"), 1));
    assert_eq!(0,orgs.len());

    let response=with_test_login(client.post("/api/orgs/Acme"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let org:Organization = response.into_json().unwrap(); 
    assert_eq!("Acme",&org.name);

    let org:Organization=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}",org.id)), 1));
    assert_eq!("Acme",&org.name);

    let response=with_test_login(client.post("/api/orgs/Acme"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let org2:Organization = response.into_json().unwrap(); 
    assert_eq!("Acme",&org2.name);
    assert_eq!(org.id,org2.id);

    let response=with_test_login(client.post("/api/orgs/Acme"), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let lt:i64=json_ok_response(with_test_login(client.get("/api/orgs/count"), 1));
    assert_eq!(1,lt);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs/all"), 1));
    assert_eq!(1,orgs.len());
    assert_eq!("Acme",&orgs[0].name);
    
    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let lt:i64=json_ok_response(with_test_login(client.get("/api/orgs/count"), 1));
    assert_eq!(0,lt);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs/all"), 1));
    assert_eq!(0,orgs.len());
}

#[test]
#[serial]
fn members(){
    let client= setup();

    let response=with_test_login(client.post("/api/orgs/Acme"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let org:Organization = response.into_json().unwrap(); 
    assert_eq!("Acme",&org.name);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert_eq!(None,ombr);

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
    assert_eq!(false,mbr.org_admin);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert_eq!(true,ombr.is_some());
    if let Some(mbr)=ombr {
        assert_eq!(org.id,mbr.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
        assert_eq!(false,mbr.org_admin);
    }

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32?admin=true",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
    assert_eq!(true,mbr.org_admin);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert_eq!(true,ombr.is_some());
    if let Some(mbr)=ombr {
        assert_eq!(org.id,mbr.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
        assert_eq!(true,mbr.org_admin);
    }

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs"), 2));
    assert_eq!(1,orgs.len());
    assert_eq!(org.id,orgs[0].id);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs/"), 1));
    assert_eq!(0,orgs.len());

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs/all"), 1));
    assert_eq!(1,orgs.len());

    let response=with_test_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32?admin=true",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    
    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs/"), 2));
    assert_eq!(0,orgs.len());

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
}