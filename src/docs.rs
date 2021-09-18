use rocket::{Route, State};
use crate::base::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::form::Form;
use rocket::fs::TempFile;
use std::path::PathBuf;
use std::fs::{read,create_dir_all, remove_file};
use crate::model::Document;
use crate::schema::documents::dsl::documents;
use crate::schema::documents::{id,owner};

use uuid::Uuid;

use chrono::Utc;
use diesel::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(FromForm)]
struct Upload<'r> {
    files: Vec<TempFile<'r>>,
}


#[post("/", data = "<upload>")]
async fn upload_doc(userid: UserId,mut upload: Form<Upload<'_>>, config: &State<Config>, conn: MainDbConn) -> DRResult<Json<Vec<String>>>{
    let mut uuids=vec![];
    println!("files:{}",upload.files.len());
    for file in upload.files.iter_mut() {
        if let Some(name) = file.name() {
            println!("name:{}",name);
            let mut full=PathBuf::new();
            full.push(&config.temp_dir.original());
            full.push(userid.0.to_string());
            create_dir_all(&full)?;
            full.push(name);
            let doc_name= file.raw_name().map(|f| format!("{}",f.dangerous_unsafe_unsanitized_raw())).unwrap_or(String::from(name));
            file.persist_to(&full).await?;
            let data=read(&full)?;
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            let hash= hasher.finish();

            let doc = Document{
                id: Uuid::new_v4(),
                name: doc_name,
                created: Utc::now(),
                owner: userid.0,
                mime: file.content_type().map(|ct| format!("{}",ct)),
                size: data.len() as i64,
                data: data,
                hash: Some(format!("{}",hash))
            };
            remove_file(&full)?;

            let doc_id=conn.run(move |c| {
                diesel::insert_into(documents)
                    .values(&doc)
                    .execute(c)
                    .map(|_| doc.id)
            }).await?;
            uuids.push(doc_id.to_string());
        }
    }
    Ok(Json(uuids))
}

#[get("/<uuid>")]
async fn get_doc(userid: UserId,uuid: &str, conn: MainDbConn) -> DRResult<Json<Document>>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    let mut docs=conn.run(move |c| {
        documents.filter(id.eq(real_uuid)).filter(owner.eq(userid.0)).load::<Document>(c)
    }).await?;
    match docs.pop(){
        None => Err(DRError::NotFoundError),
        Some(doc) =>  Ok(Json(doc)),
    }
    
}

#[delete("/<uuid>")]
async fn delete_doc(userid: UserId,uuid: &str, conn: MainDbConn) -> DRResult<Status>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    conn.run(move |c| {
        diesel::delete(documents.filter(id.eq(real_uuid)).filter(owner.eq(userid.0))).execute(c)
    }).await?;
    Ok(Status::NoContent)
    
}

pub fn routes() -> Vec<Route> {
    routes![upload_doc, get_doc, delete_doc]
}