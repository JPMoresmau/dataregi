mod common;

use std::fs;
use common::{setup,with_test_login};
use rocket::http::{ContentType, Status, Header};

#[test]
fn upload_get_delete() {
    let client= setup();

    let file = fs::read("test_data/1sheet1cell.ods").unwrap();
    let mut cnt=vec![];

    cnt.extend("-----------------------------3511489321811197009899980000\r\n".as_bytes());    
    cnt.extend("Content-Disposition: form-data; name=\"files\"; filename=\"1sheet1cell.ods\"\r\n".as_bytes());
    //cnt.extend("Content-Transfer-Encoding: binary\r\n".as_bytes());
    cnt.extend("Content-Type: application/vnd.oasis.opendocument.spreadsheet\r\n\r\n".as_bytes());
    cnt.extend(&file);
    cnt.extend("\r\n-----------------------------3511489321811197009899980000--\r\n".as_bytes()); 

    let req = with_test_login(client.post("/docs"))
        .header(ContentType::with_params("multipart", "form-data", ("boundary", "---------------------------3511489321811197009899980000")))
        .header(Header::new("Content-Length",format!("{}",cnt.len())))
        .body(cnt);
    
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

}   