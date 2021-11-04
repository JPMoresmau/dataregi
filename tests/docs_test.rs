mod common;

use dataregi::{docs::DocumentUpload, model::{Document,DocumentInfo}};
use std::fs;
use common::{setup,with_test_login, do_upload,upload,delete,json_ok_response};
use rocket::http::{ContentType, Status};
use serial_test::serial;


#[test]
#[serial]
fn upload_get_delete() {
    let client= setup();

    // upload doc
    let file = fs::read("test_data/1sheet1cell.ods").unwrap();
    let uuid = upload(&client, "test_data/1sheet1cell.ods");
   
    // read doc again
    let doc:Document = json_ok_response(with_test_login(client.get(format!("/api/docs/{}",uuid)), 1));
    assert_eq!(uuid,doc.id);
    assert_eq!("b9518d55-3256-4b96-81d0-65b1d7c4fb31",doc.owner.to_string());
    assert_eq!(file,doc.data);
    assert_eq!(file.len() as i64, doc.size);

    // read metadata only
    let doc:DocumentInfo = json_ok_response(with_test_login(client.get(format!("/api/docs/{}/info",uuid)), 1));
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

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs/"), 1));
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

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?limit=2"), 1));
    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
   
    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");
   
    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?limit=2&order=recent"), 1));
    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
   
    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");
   

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?limit=2&order=name"), 1));
    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid1);
    assert_eq!(doci2.id,uuid3);
   
    assert_eq!(doci1.name,"1sheet1cell.ods");
    assert_eq!(doci2.name,"1sheet1col.ods");

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?limit=2&order=name&offset=1"), 1));
    assert_eq!(2,docs.len());

    let doci1=&docs[0];
    let doci2=&docs[1];
   
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
   
    assert_eq!(doci1.name,"1sheet1col.ods");
    assert_eq!(doci2.name,"1sheet1row.ods");

    delete(&client, &vec![uuid1,uuid2,uuid3]);

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs/"), 1));
    assert_eq!(0,docs.len());
}

#[test]
#[serial]
fn search(){
    let client= setup();

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs/"), 1));
    assert_eq!(0,docs.len());

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid2 = upload(&client, "test_data/1sheet1row.ods");
    let uuid3 = upload(&client, "test_data/1sheet1col.ods");

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?order=name&name=1sheet1cell.ods"), 1));
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    assert_eq!(doci1.id,uuid1);
   
    assert_eq!(doci1.name,"1sheet1cell.ods");

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?order=name&name=sheet"), 1));
    assert_eq!(0,docs.len());

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?order=name&name=*sheet*"), 1));
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

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs?order=name&name=*SHEET*"), 1));
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

    let docs:Vec<DocumentInfo> = json_ok_response(with_test_login(client.get("/api/docs/"), 1));
    assert_eq!(0,docs.len());
}

#[test]
#[serial]
fn count(){
    let client= setup();

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(0,cnt);

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(1,cnt);

    let uuid2 = upload(&client, "test_data/1sheet1row.ods");

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(2,cnt);

    let uuid3 = upload(&client, "test_data/1sheet1col.ods");

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(3,cnt);

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true&name=*col*"), 1));
    assert_eq!(1,cnt);

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true&name=*COL*"), 1));
    assert_eq!(1,cnt);


    delete(&client, &vec![uuid1,uuid2,uuid3]);
    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(0,cnt);
}

#[test]
#[serial]
fn distinct(){
    let client= setup();

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(0,cnt);

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true&distinct=true"), 1));
    assert_eq!(0,cnt);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/"), 1));
    assert_eq!(0,docs.len());

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?distinct=true"), 1));
    assert_eq!(0,docs.len());

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");
    let uuid2 = upload(&client, "test_data/v2/1sheet1cell.ods");
    let uuid3 = upload(&client, "test_data/1sheet1row.ods");

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true"), 1));
    assert_eq!(3,cnt);

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true&distinct=true"), 1));
    assert_eq!(2,cnt);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/"), 1));
    assert_eq!(3,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    let doci3=&docs[2];

    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);
    assert_eq!(doci3.id,uuid1);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?distinct=true"), 1));
    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid3);
    assert_eq!(doci2.id,uuid2);

    delete(&client, &vec![uuid1]);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?distinct=true"), 1));
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

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?name=1sheet1cell.ods"), 1));
    assert_eq!(2,docs.len());
    let doci1=&docs[0];
    let doci2=&docs[1];
    
    assert_eq!(doci1.id,uuid2);
    assert_eq!(doci2.id,uuid1);

    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get("/api/docs/?1sheet1cell.ods&distinct=true"), 1));
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid2);
    
    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get(format!("/api/docs/?name=1sheet1cell.ods&distinct=true&except={}",uuid1)), 1));
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid2);
    
    let docs:Vec<DocumentInfo> =json_ok_response(with_test_login(client.get(format!("/api/docs/?name=1sheet1cell.ods&distinct=true&except={}",uuid2)), 1));
    assert_eq!(1,docs.len());
    let doci1=&docs[0];
    
    assert_eq!(doci1.id,uuid1);

    let cnt:i64 = json_ok_response(with_test_login(client.get("/api/docs/count?owner=true&name=1sheet1cell.ods"), 1));
    assert_eq!(2,cnt);

    let cnt:i64 = json_ok_response(with_test_login(client.get(format!("/api/docs/count?owner=true&name=1sheet1cell.ods&except={}",uuid2)), 1));
    assert_eq!(1,cnt);

    delete(&client, &vec![uuid1,uuid2]);
}

#[test]
#[serial]
fn detect_duplicate(){
    let client= setup();

    let uuid1 = upload(&client, "test_data/1sheet1cell.ods");

    let upd = do_upload(&client, "test_data/1sheet1cell.ods");
    match upd {
        DocumentUpload::AlreadyExists{upload_name, existing_id}=> {
            assert_eq!(uuid1,existing_id);
            assert_eq!("1sheet1cell.ods",&upload_name);
        },
        du => panic!("Unexpect upload: {:?}",du),
    };
    delete(&client, &vec![uuid1]);
}