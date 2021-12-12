use dataregi::{model::{Limit},docs::DocumentUpload};
use crate::common::{setup,with_test_login,do_upload,upload,delete, json_ok_response};
use rocket::http::Status;
use serial_test::serial;

#[test]
#[serial]
fn crud() {
    let client= setup();

    let lt:Limit=json_ok_response(with_test_login(client.get("/api/limits"), 1));
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",&lt.user_id.to_string());

    assert_eq!(100,lt.max_documents);
    assert_eq!(1048576000,lt.max_size);
    assert_eq!(0,lt.current_documents);
    assert_eq!(0,lt.current_size);

    let lt:Limit=json_ok_response(with_test_login(client.get("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb34"), 1));
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb34",&lt.user_id.to_string());

    assert_eq!(1,lt.max_documents);
    assert_eq!(8000,lt.max_size);
    assert_eq!(0,lt.current_documents);
    assert_eq!(0,lt.current_size);

    let cnt="{\"max_documents\":2,\"max_size\":9000}";
    let req = with_test_login(client.put("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb34"), 1)
        .body(cnt);
    
    let response = req.dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let lt:Limit=json_ok_response(with_test_login(client.get("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb34"), 1));
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb34",&lt.user_id.to_string());

    assert_eq!(2,lt.max_documents);
    assert_eq!(9000,lt.max_size);
    assert_eq!(0,lt.current_documents);
    assert_eq!(0,lt.current_size);

    let cnt="{\"max_documents\":1,\"max_size\":8000}";
    let req = with_test_login(client.put("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb34"), 1)
        .body(cnt);
    
    let response = req.dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let lt:Limit=json_ok_response(with_test_login(client.get("/api/limits"), 4));
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb34",&lt.user_id.to_string());
    assert_eq!(1,lt.max_documents);
    assert_eq!(8000,lt.max_size);
    assert_eq!(0,lt.current_documents);
    assert_eq!(0,lt.current_size);

}


#[test]
#[serial]
fn forbidden() {
    let client= setup();

    let response=with_test_login(client.get("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb34"), 2).dispatch();
    assert_eq!(response.status(),Status::Forbidden);

    let cnt="{\"max_documents\":1,\"max_size\":8000}";
    let req = with_test_login(client.put("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb34"), 2)
        .body(cnt);
    
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Forbidden);
}


#[test]
#[serial]
fn enforced() {
    let client= setup();

    let cnt="{\"max_documents\":1,\"max_size\":25000}";
    let req = with_test_login(client.put("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb31"), 1)
        .body(cnt);
    let response = req.dispatch();
    assert_eq!(response.status(),Status::NoContent);

    let uuid = upload(&client, "test_data/1sheet1cell.ods");

    let lt:Limit=json_ok_response(with_test_login(client.get("/api/limits"), 1));
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",&lt.user_id.to_string());

    assert_eq!(1,lt.max_documents);
    assert_eq!(25000,lt.max_size);
    assert_eq!(1,lt.current_documents);
    assert_eq!(7651,lt.current_size);

    // too many docs
    let du= do_upload(&client, "test_data/1sheet1col.ods", 1);  
    assert_eq!(DocumentUpload::LimitsReached,du);

    delete(&client,&[uuid]);

    let cnt="{\"max_documents\":1,\"max_size\":8000}";
    let req = with_test_login(client.put("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb31"), 1)
        .body(cnt);
    let response = req.dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // too big
    let du= do_upload(&client, "test_data/1sheet1col.ods", 1);  
    assert_eq!(DocumentUpload::LimitsReached,du);

    let cnt="{\"max_documents\":100,\"max_size\":1048576000}";
    let req = with_test_login(client.put("/api/limits/b9518d55-3256-4b96-81d0-65b1d7c4fb31"), 1)
        .body(cnt);
    let response = req.dispatch();
    assert_eq!(response.status(),Status::NoContent);
}