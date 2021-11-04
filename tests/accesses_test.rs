mod common;

use dataregi::{model::{User}};
use common::{setup,with_test_login,upload,delete, json_ok_response};
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
    assert_eq!(response.status(),Status::Ok);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1)); 
    assert_eq!(1,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(1,users_cnt);

    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&users[0].id.to_string());

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(2,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(2,users_cnt);

    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(1,users.len());

    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&users[0].id.to_string());

    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}",uuid)), 1));
    assert_eq!(0,users.len());

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(0,users_cnt);

    delete(&client, &vec![uuid]);
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
    assert_eq!(response.status(),Status::Ok);

    // cannot delete if no access (user: 3)
    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 3).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    // can add if access but not owner
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 2).dispatch();
    assert_eq!(response.status(),Status::Ok);

    // can remove ourselves
    let response= with_test_login(client.delete(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 2).dispatch();
    assert_eq!(response.status(),Status::Ok);
    
    // delete document should delete accesses
    delete(&client, &vec![uuid]);
}

#[test]
#[serial]
fn paging_accesses() {
    let client= setup();

    let uuid = upload(&client, "test_data/1sheet1cell.ods");

    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb32")), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb33")), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    let response= with_test_login(client.put(format!("/api/accesses/{}/{}",uuid,"b9518d55-3256-4b96-81d0-65b1d7c4fb34")), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);

    let users_cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}/count",uuid)), 1)); 
    assert_eq!(3,users_cnt);

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}?limit=2",uuid)), 1));
    assert_eq!(2,users.len());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb32",&users[0].id.to_string());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb33",&users[1].id.to_string());

    let users:Vec<User> = json_ok_response(with_test_login(client.get(format!("/api/accesses/{}?limit=2&offset=2",uuid)), 1));
    assert_eq!(1,users.len());
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb34",&users[0].id.to_string());
   
    delete(&client, &vec![uuid]);
}