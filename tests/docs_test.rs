mod common;

use dataregi::model::Document;
use std::fs;
use common::{setup,with_test_login};
use rocket::http::{ContentType, Status, Header};
use uuid::Uuid;

#[test]
fn upload_get_delete() {
    let client= setup();

    // upload doc
    let file = fs::read("test_data/1sheet1cell.ods").unwrap();
    let mut cnt=vec![];

    cnt.extend("-----------------------------3511489321811197009899980000\r\n".as_bytes());    
    cnt.extend("Content-Disposition: form-data; name=\"files\"; filename=\"1sheet1cell.ods\"\r\n".as_bytes());
    //cnt.extend("Content-Transfer-Encoding: binary\r\n".as_bytes());
    cnt.extend("Content-Type: application/vnd.oasis.opendocument.spreadsheet\r\n\r\n".as_bytes());
    cnt.extend(&file);
    cnt.extend("\r\n-----------------------------3511489321811197009899980000--\r\n".as_bytes()); 

    let req = with_test_login(client.post("/docs"), 1)
        .header(ContentType::with_params("multipart", "form-data", ("boundary", "---------------------------3511489321811197009899980000")))
        .header(Header::new("Content-Length",format!("{}",cnt.len())))
        .body(cnt);
    
    let response = req.dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    // get id
    let uuids:Vec<Uuid> = response.into_json().unwrap(); 

    assert_eq!(uuids.len(),1);
    
    // read doc again
    let response= with_test_login(client.get(format!("/docs/{}",uuids[0])), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let doc:Document = response.into_json().unwrap(); 

    assert_eq!(uuids[0],doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(file,doc.data);
    assert_eq!(file.len() as i64, doc.size);

    // read as another user, not found
    let response= with_test_login(client.get(format!("/docs/{}",uuids[0])), 2).dispatch();
    assert_eq!(response.status(),Status::NotFound);

    // delete as another user, no error but no effect
    let response= with_test_login(client.delete(format!("/docs/{}",uuids[0])), 2).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // still there
    let response= with_test_login(client.get(format!("/docs/{}",uuids[0])), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    // delete
    let response= with_test_login(client.delete(format!("/docs/{}",uuids[0])), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // not here anymore
    let response= with_test_login(client.get(format!("/docs/{}",uuids[0])), 1).dispatch();
    assert_eq!(response.status(),Status::NotFound);
    let response= with_test_login(client.get(format!("/docs/{}",uuids[0])), 2).dispatch();
    assert_eq!(response.status(),Status::NotFound);
}