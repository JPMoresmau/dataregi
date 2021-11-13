use dataregi::{model::{Organization, Member, MemberInfo, Document, DocumentInfo}};
use crate::common::{setup,with_test_login,with_org_login,upload_org,delete, json_ok_response};
use rocket::http::{ContentType, Status};
use serial_test::serial;

#[test]
#[serial]
fn crud() {
    let client= setup();

    let lt:i64=json_ok_response(with_test_login(client.get("/api/orgs/count?member=false"), 1));
    assert_eq!(0,lt);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=false"), 1));
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

    let lt:i64=json_ok_response(with_test_login(client.get("/api/orgs/count?member=false"), 1));
    assert_eq!(1,lt);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=false"), 1));
    assert_eq!(1,orgs.len());
    assert_eq!("Acme",&orgs[0].name);
    
    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let lt:i64=json_ok_response(with_test_login(client.get("/api/orgs/count?member=false"), 1));
    assert_eq!(0,lt);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=false"), 1));
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

    let response=with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
    assert!(!mbr.org_admin);

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert!(ombr.is_some());
    if let Some(mbr)=ombr {
        assert_eq!(org.id,mbr.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
        assert!(!mbr.org_admin);
    }

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32?admin=true",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
    assert!(mbr.org_admin);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert!(ombr.is_some());
    if let Some(mbr)=ombr {
        assert_eq!(org.id,mbr.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
        assert!(mbr.org_admin);
    }

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=true"), 2));
    assert_eq!(1,orgs.len());
    assert_eq!(org.id,orgs[0].id);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=true"), 1));
    assert_eq!(0,orgs.len());

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=false"), 1));
    assert_eq!(1,orgs.len());

    let response=with_test_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    
    let response=with_test_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=true"), 2));
    assert_eq!(0,orgs.len());

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
}

#[test]
#[serial]
fn members_email(){
    let client= setup();

    let response=with_test_login(client.post("/api/orgs/Acme"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let org:Organization = response.into_json().unwrap(); 
    assert_eq!("Acme",&org.name);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert_eq!(None,ombr);

    let response=with_test_login(client.put(format!("/api/orgs/{}/test2@dataregi.com",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
    assert!(!mbr.org_admin);

   
    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert!(ombr.is_some());
    if let Some(mbr)=ombr {
        assert_eq!(org.id,mbr.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
        assert!(!mbr.org_admin);
    }

    
    let response=with_test_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    
    let response=with_test_login(client.put(format!("/api/orgs/{}/test5@dataregi.com",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert!(!mbr.org_admin);
    
    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/{}",&org.id,mbr.user_id)), 1));
    assert!(ombr.is_some());
    if let Some(mbr2)=ombr {
        assert_eq!(org.id,mbr2.org_id);
        assert_eq!(mbr.user_id,mbr.user_id);
        assert!(!mbr2.org_admin);
    }

    let response=with_test_login(client.delete(format!("/api/orgs/{}/{}",&org.id,mbr.user_id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let orgs:Vec<Organization>=json_ok_response(with_test_login(client.get("/api/orgs?member=true"), 2));
    assert_eq!(0,orgs.len());

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
}


#[test]
#[serial]
fn members_org_admin(){
    let client= setup();

    let response=with_test_login(client.post("/api/orgs/Acme"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let org:Organization = response.into_json().unwrap(); 
    assert_eq!("Acme",&org.name);

    let ombr:Option<Member>=json_ok_response(with_test_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1));
    assert_eq!(None,ombr);

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32?admin=true",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr.user_id.to_string());
    assert!(mbr.org_admin);

    let v=vec![mbr];
    let response=with_org_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb33",&org.id)), 2, &v).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let ombr:Option<Member>=json_ok_response(with_org_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb33",&org.id)), 2, &v));
    assert!(ombr.is_some());
    if let Some(mbr2)=ombr {
        assert_eq!(org.id,mbr2.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&mbr2.user_id.to_string());
        assert!(!mbr2.org_admin);
    }

    let response=with_org_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb33?admin=true",&org.id)), 2, &v).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&mbr.user_id.to_string());
    assert!(mbr.org_admin);

    let ombr:Option<Member>=json_ok_response(with_org_login(client.get(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb33",&org.id)), 2, &v));
    assert!(ombr.is_some());
    if let Some(mbr2)=ombr {
        assert_eq!(org.id,mbr2.org_id);
        assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&mbr2.user_id.to_string());
        assert!(mbr2.org_admin);
    }

    let cnt:i64=json_ok_response(with_org_login(client.get(format!("/api/orgs/{}/members/count",&org.id)), 2, &v));
    assert_eq!(2,cnt);

    let mbrs:Vec<MemberInfo> = json_ok_response(with_org_login(client.get(format!("/api/orgs/{}/members",&org.id)), 2, &v));
    assert_eq!(2,mbrs.len());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbrs[0].user_id.to_string());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&mbrs[1].user_id.to_string());
    assert_eq!("Test User 2",&mbrs[0].name);
    assert_eq!("Test User 3",&mbrs[1].name);
    assert_eq!("test2@dataregi.com",&mbrs[0].email);
    assert_eq!("test3@dataregi.com",&mbrs[1].email);

    let response=with_org_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb33",&org.id)), 2, &v).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    
    let response=with_test_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
}

#[test]
#[serial]
fn org_documents(){
    let client= setup();
 
    let response=with_test_login(client.post("/api/orgs/Acme"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let org:Organization = response.into_json().unwrap(); 
    assert_eq!("Acme",&org.name);

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb31?admin=true",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",&mbr.user_id.to_string());
    assert!(mbr.org_admin);

    let mut v=vec![mbr];

    let response=with_test_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let mbr2:Member = response.into_json().unwrap(); 
    assert_eq!(org.id,mbr2.org_id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&mbr2.user_id.to_string());
    assert!(!mbr2.org_admin);

    let v2=vec![mbr2];

    let response=with_org_login(client.put(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1, &v).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let uuid = upload_org(&client, "test_data/1sheet1cell.ods",v.pop().unwrap());

    // as owner
    // read doc again
    let doc:Document = json_ok_response(with_test_login(client.get(format!("/api/docs/{}",uuid)), 1));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(Some(org.id),doc.org_id);
    
    // read metadata only
    let doc:DocumentInfo = json_ok_response(with_test_login(client.get(format!("/api/docs/{}/info",uuid)), 1));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(Some(org.id),doc.org_id);

    // as member of org
    // read doc again
    let doc:Document = json_ok_response(with_org_login(client.get(format!("/api/docs/{}",uuid)), 2, &v2));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(Some(org.id),doc.org_id);

    // read metadata only
    let doc:DocumentInfo = json_ok_response(with_org_login(client.get(format!("/api/docs/{}/info",uuid)), 2, &v2));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(Some(org.id),doc.org_id);

    let cnt:i64 = json_ok_response(with_org_login(client.get("/api/docs/count"), 2, &v2));
    assert_eq!(1,cnt);


    delete(&client, &[uuid]);

    let response=with_org_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb32",&org.id)), 1, &v).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    
    let response=with_test_login(client.delete(format!("/api/orgs/{}/b9518d55-3256-4b96-81d0-65b1d7c4fb31",&org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    

    let response=with_test_login(client.delete(format!("/api/orgs/{}",org.id)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

}