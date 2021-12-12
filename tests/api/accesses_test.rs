use dataregi::{model::{User,Document,DocumentInfo}};
use crate::common::{setup,with_test_login,upload, upload_user,delete,delete_user, json_ok_response};
use rocket::http::{ContentType, Status};
use serial_test::serial;

#[test]
#[serial]
fn crud() {
    let client= setup();

    let uuid = upload(&client, "test_data/1sheet1cell.ods");

    let response= with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let users:Vec<User> = response.into_json().unwrap(); 

    assert_eq!(0,users.len());

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1)); 
    assert_eq!(1,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(1,users_cnt);

    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&users[0].id.to_string());

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(2,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(2,users_cnt);

    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(1,users.len());

    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&users[0].id.to_string());

    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(0,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(0,users_cnt);

    delete(&client, &[uuid]);
}

#[test]
#[serial]
fn add_email() {
    let client= setup();

    let uuid = upload(&client, "test_data/1sheet1cell.ods");

    let response= with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let users:Vec<User> = response.into_json().unwrap(); 

    assert_eq!(0,users.len());

    let response= with_test_login(client.post(format!("/api/accesses/{}/{}",uuid,"test2@dataregi.com")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1)); 
    assert_eq!(1,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(1,users_cnt);

    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&users[0].id.to_string());

    let response= with_test_login(client.post(format!("/api/accesses/{}/{}",uuid,"test0@dataregi.com")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(2,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(2,users_cnt);
    
    delete(&client, &[uuid]);
}


#[test]
#[serial]
fn forbidden_accesses() {
    let client= setup();

    let uuid = upload(&client, "test_data/1sheet1cell.ods");

    // cannot add if no access (user: 2)
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // cannot delete if no access (user: 3)
    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 3).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    // can add if access but not owner
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 2).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // can remove ourselves
    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 2).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    
    // delete document should delete accesses
    delete(&client, &[uuid]);
}

#[test]
#[serial]
fn paging_accesses() {
    let client= setup();

    let uuid = upload(&client, "test_data/1sheet1cell.ods");

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb34")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(3,users_cnt);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}?limit=2",uuid)), 1));
    assert_eq!(2,users.len());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&users[0].id.to_string());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&users[1].id.to_string());

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}?limit=2&offset=2",uuid)), 1));
    assert_eq!(1,users.len());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb34",&users[0].id.to_string());
   
    delete(&client, &[uuid]);
}

#[test]
#[serial]
fn get_docs_via_access() {
    let client= setup();

    let uuid = upload(&client, "test_data/1sheet1cell.ods");
   
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let doc:Document = json_ok_response(with_test_login(client.get(format!("/api/docs/{}",uuid)), 2));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());

    let doc:DocumentInfo = json_ok_response(with_test_login(client.get(format!("/api/docs/{}/info",uuid)), 1));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs/"), 2));
    assert_eq!(1,docs.len());

    let doci1=&docs[0];
    assert_eq!(doci1.id,uuid);
  
    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=false"), 2));
    assert_eq!(1,cnt);

    // delete as another user, no error but no effect
    let response= with_test_login(client.delete(format!("/api/docs/{}",uuid)), 2).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs/"), 2));
    assert_eq!(1,docs.len());

    delete(&client, &[uuid]);

}

#[test]
#[serial]
fn get_versions_via_access() {
    let client= setup();

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid1,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let uuid2 = upload_user(&client, "test_data/v2/1sheet1cell.ods",2);

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid2,"b9518d55-3256-4b96-81d0-65b1d7c4fb31")), 2).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?name=1sheet1cell.ods"), 1));
    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid2);
    assert_eq!(doci2.id,uuid1);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?name=1sheet1cell.ods"), 2));
    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid2);
    assert_eq!(doci2.id,uuid1);

    delete(&client, &[uuid1]);
    delete_user(&client, &[uuid2],2);
}

#[test]
#[serial]
fn get_versions_no_access() {
    let client= setup();

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
   
    let uuid2 = upload_user(&client, "test_data/v2/1sheet1cell.ods",2);
   
    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?name=1sheet1cell.ods"), 1));
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid1);
    
    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?name=1sheet1cell.ods"), 2));
    assert_eq!(1,docs.len());
    let doci2=&docs[0];
    
    assert_eq!(doci2.id,uuid2);

    delete(&client, &[uuid1]);
    delete_user(&client, &[uuid2],2);
}