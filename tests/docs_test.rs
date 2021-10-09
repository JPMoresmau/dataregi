mod common;

use dataregi::model::{Document,DocumentInfo};
use std::fs;
use common::{setup,with_test_login};
use rocket::http::{ContentType, Status, Header};
use uuid::Uuid;
use rocket::local::blocking::Client;
use serial_test::serial;

fn upload(client: &Client, path: &str) -> Uuid {
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
    let uuids:Vec<Uuid> = response.into_json().unwrap(); 

    assert_eq!(uuids.len(),1);
    uuids[0]
}

fn delete(client: &Client, uuids: &[Uuid]) {
    for uuid in uuids.iter(){
        let response= with_test_login(client.delete(format!("/api/docs/{}",uuid)), 1).dispatch();
        assert_eq!(response.status(),Status::NoContent);
    }
}

#[test]
#[serial]
fn upload_get_delete() {
    let client= setup();

    // upload doc
    let file = fs::read("test_data/1sheet1cell.ods").unwrap();
    let uuid = upload(&client, "test_data/1sheet1cell.ods");
   
    // read doc again
    let response= with_test_login(client.get(format!("/api/docs/{}",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let doc:Document = response.into_json().unwrap(); 

    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(file,doc.data);
    assert_eq!(file.len() as i64, doc.size);

    // read metadata only
    let response= with_test_login(client.get(format!("/api/docs/{}/info",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let doc:DocumentInfo = response.into_json().unwrap(); 

    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(file.len() as i64, doc.size);

    // read as another user, not found
    let response= with_test_login(client.get(format!("/api/docs/{}",uuid)), 2).dispatch();
    assert_eq!(response.status(),Status::NotFound);

    // delete as another user, no error but no effect
    let response= with_test_login(client.delete(format!("/api/docs/{}",uuid)), 2).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // still there
    let response= with_test_login(client.get(format!("/api/docs/{}",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    // delete
    let response= with_test_login(client.delete(format!("/api/docs/{}",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::NoContent);

    // not here anymore
    let response= with_test_login(client.get(format!("/api/docs/{}",uuid)), 1).dispatch();
    assert_eq!(response.status(),Status::NotFound);
    let response= with_test_login(client.get(format!("/api/docs/{}",uuid)), 2).dispatch();
    assert_eq!(response.status(),Status::NotFound);
}

#[test]
#[serial]
fn list(){
    let client= setup();

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid2 = upload(&client, "test_data/1sheet1row.ods");
    let uuid3 = upload(&client, "test_data/1sheet1col.ods");

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(3,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
    let doci3=&docs[2];

    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
    assert_eq!(doci3.id,uuid1);

    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");
    assert_eq!(doci3.name,"1sheet1cell.ods");

    let response= with_test_login(client.get("/api/docs?limit=2"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
   
    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");
   
    let response= with_test_login(client.get("/api/docs?limit=2&order=recent"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
   
    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");
   

    let response= with_test_login(client.get("/api/docs?limit=2&order=name"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid1);
    assert_eq!(doci2.id,uuid3);
   
    assert_eq!(doci1.name,"1sheet1cell.ods");
    assert_eq!(doci2.name,"1sheet1col.ods");

    let response= with_test_login(client.get("/api/docs?limit=2&order=name&offset=1"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
   
    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");

    delete(&client, &vec![uuid1,uuid2,uuid3]);

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());
}

#[test]
#[serial]
fn search(){
    let client= setup();

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid2 = upload(&client, "test_data/1sheet1row.ods");
    let uuid3 = upload(&client, "test_data/1sheet1col.ods");

    let response= with_test_login(client.get("/api/docs?order=name&name=1sheet1cell.ods"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    assert_eq!(doci1.id,uuid1);
   
    assert_eq!(doci1.name,"1sheet1cell.ods");

    let response= with_test_login(client.get("/api/docs?order=name&name=sheet"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());

    let response= with_test_login(client.get("/api/docs?order=name&name=*sheet*"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(3,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    let doci3=&docs[2];

    assert_eq!(doci1.id,uuid1);
    assert_eq!(doci2.id,uuid3);
    assert_eq!(doci3.id,uuid2);

    assert_eq!(doci1.name,"1sheet1cell.ods");
    assert_eq!(doci2.name,"1sheet1col.ods");
    assert_eq!(doci3.name,"1sheet1row.ods");

    let response= with_test_login(client.get("/api/docs?order=name&name=*SHEET*"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(3,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    let doci3=&docs[2];

    assert_eq!(doci1.id,uuid1);
    assert_eq!(doci2.id,uuid3);
    assert_eq!(doci3.id,uuid2);

    assert_eq!(doci1.name,"1sheet1cell.ods");
    assert_eq!(doci2.name,"1sheet1col.ods");
    assert_eq!(doci3.name,"1sheet1row.ods");

    delete(&client, &vec![uuid1,uuid2,uuid3]);

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());
}

#[test]
#[serial]
fn count(){
    let client= setup();

    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(0,cnt);

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");

    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(1,cnt);

    let uuid2 = upload(&client, "test_data/1sheet1row.ods");

    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(2,cnt);

    let uuid3 = upload(&client, "test_data/1sheet1col.ods");

    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(3,cnt);

    let response= with_test_login(client.get("/api/docs/count?owner=true&name=*col*"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(1,cnt);

    let response= with_test_login(client.get("/api/docs/count?owner=true&name=*COL*"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(1,cnt);


    delete(&client, &vec![uuid1,uuid2,uuid3]);
    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(0,cnt);
}

#[test]
#[serial]
fn distinct(){
    let client= setup();

    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(0,cnt);

    let response= with_test_login(client.get("/api/docs/count?owner=true&distinct=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(0,cnt);

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());

    let response= with_test_login(client.get("/api/docs/?distinct=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(0,docs.len());

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid2 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid3 = upload(&client, "test_data/1sheet1row.ods");

    let response= with_test_login(client.get("/api/docs/count?owner=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(3,cnt);

    let response= with_test_login(client.get("/api/docs/count?owner=true&distinct=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(2,cnt);

    let response= with_test_login(client.get("/api/docs/"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(3,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    let doci3=&docs[2];

    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
    assert_eq!(doci3.id,uuid1);

    let response= with_test_login(client.get("/api/docs/?distinct=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);

    delete(&client, &vec![uuid1]);

    let response= with_test_login(client.get("/api/docs/?distinct=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 

    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);

    delete(&client, &vec![uuid2,uuid3]);
}

#[test]
#[serial]
fn versions(){
    let client= setup();

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid2 = upload(&client, "test_data/v2/1sheet1cell.ods");

    let response= with_test_login(client.get("/api/docs/?name=1sheet1cell.ods"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 
    
    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid2);
    assert_eq!(doci2.id,uuid1);

    let response= with_test_login(client.get("/api/docs/?1sheet1cell.ods&distinct=true"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 
    
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid2);
    
    let response= with_test_login(client.get(format!("/api/docs/?name=1sheet1cell.ods&distinct=true&except={}",uuid1)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 
    
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid2);
    
    let response= with_test_login(client.get(format!("/api/docs/?name=1sheet1cell.ods&distinct=true&except={}",uuid2)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let docs:Vec<DocumentInfo> = response.into_json().unwrap(); 
    
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid1);

    let response= with_test_login(client.get("/api/docs/count?owner=true&name=1sheet1cell.ods"), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(2,cnt);

    let response= with_test_login(client.get(format!("/api/docs/count?owner=true&name=1sheet1cell.ods&except={}",uuid2)), 1).dispatch();
    assert_eq!(response.status(),Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let cnt:i64 = response.into_json().unwrap(); 

    assert_eq!(1,cnt);

    delete(&client, &vec![uuid1,uuid2]);
}