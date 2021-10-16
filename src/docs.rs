use rocket::{Route, State};
use crate::base::*;
use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use rocket::form::Form;
use rocket::fs::TempFile;
use std::str::FromStr;
use std::path::PathBuf;
use std::fs::{read,create_dir_all, remove_file};
use crate::model::{Document,DocumentInfo};
use crate::schema::documents::dsl::documents;
use crate::schema::documents as docs;
use rocket::serde::{Deserialize,Serialize};

use uuid::Uuid;

use chrono::Utc;
use diesel::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(FromForm)]
struct Upload<'r> {
    files: Vec<TempFile<'r>>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum DocumentUpload {
    Ok{id: Uuid},
    AlreadyExists{
        upload_name: String,
        existing_id: Uuid,
    },
}

#[post("/", data = "<upload>")]
async fn upload_doc(userid: UserId,mut upload: Form<Upload<'_>>, config: &State<Config>, conn: MainDbConn) -> DRResult<Json<Vec<DocumentUpload>>> {
    let mut uuids=vec![];
    println!("files:{}",upload.files.len());
    for file in upload.files.iter_mut() {
        if let Some(name) = file.name() {
            println!("name:{}",name);
            let mut full=PathBuf::new();
            full.push(&config.temp_dir.original());
            full.push(&userid.0.to_string());
            create_dir_all(&full)?;
            full.push(name);
            let doc_name= file.raw_name().map(|f| format!("{}",f.dangerous_unsafe_unsanitized_raw())).unwrap_or(String::from(name));
            
            let short_name=  match doc_name.rfind('/') {
                Some(ix) => String::from(doc_name.split_at(ix+1).1),
                _ => doc_name,
            };
            file.persist_to(&full).await?;
            let data=read(&full)?;
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            let hash= hasher.finish().to_string();

            let uid=userid.0.clone();
            let h = hash.clone();
            let sn=short_name.clone();
            let mut docs=conn.run(move |c| {
                documents.filter(docs::hash.eq(Some(h))).filter(docs::name.eq(sn)).filter(docs::owner.eq(uid))
                .select(docs::id)
                .load::<Uuid>(c)
            }).await?;
            if let Some(doc)=docs.pop() {
                uuids.push(DocumentUpload::AlreadyExists{upload_name:short_name, existing_id:doc});
            } else {

                let doc = Document{
                    id: Uuid::new_v4(),
                    name: short_name,
                    created: Utc::now(),
                    owner: userid.0,
                    mime: file.content_type().map(|ct| format!("{}",ct)),
                    size: data.len() as i64,
                    data: data,
                    hash: Some(hash)
                };
                
                let doc_id=conn.run(move |c| {
                    diesel::insert_into(documents)
                        .values(&doc)
                        .execute(c)
                        .map(|_| doc.id)
                }).await?;
                uuids.push(DocumentUpload::Ok{id:doc_id});
            } 
            remove_file(&full)?;
        }
    }
    Ok(Json(uuids))
}

#[get("/<uuid>")]
async fn get_doc(userid: UserId,uuid: &str, conn: MainDbConn) -> DRResult<Json<Document>>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    let mut docs=conn.run(move |c| {
        documents.filter(docs::id.eq(real_uuid)).filter(docs::owner.eq(userid.0)).load::<Document>(c)
    }).await?;
    match docs.pop(){
        None => Err(StructuredError::not_found("Document not found")),
        Some(doc) =>  Ok(Json(doc)),
    }
    
}

#[get("/<uuid>/info")]
async fn get_doc_info(userid: UserId,uuid: &str, conn: MainDbConn) -> DRResult<Json<DocumentInfo>>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    let mut docs=conn.run(move |c| {
        documents.filter(docs::id.eq(real_uuid)).filter(docs::owner.eq(userid.0))
            .select((docs::id,docs::name,docs::created,docs::owner,docs::mime,docs::size))
            .load::<DocumentInfo>(c)
    }).await?;
    match docs.pop(){
        None => Err(StructuredError::not_found("Document not found")),
        Some(doc) =>  Ok(Json(doc)),
    }
    
}

#[get("/<uuid>/data")]
async fn get_doc_data(userid: UserId,uuid: &str, conn: MainDbConn) -> DRResult<Download>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    let mut docs=conn.run(move |c| {
        documents.filter(docs::id.eq(real_uuid)).filter(docs::owner.eq(userid.0))
            .load::<Document>(c)
    }).await?;
    match docs.pop(){
        None => Err(StructuredError::not_found("Document not found")),
        Some(doc) =>  Ok(Download{
            content_type: doc.mime.map(|s| ContentType::from_str(&s).ok()).flatten().unwrap_or_else(|| ContentType::Binary)
            ,filename: doc.name
            ,data: doc.data}),
    }
    
}

#[delete("/<uuid>")]
async fn delete_doc(userid: UserId,uuid: &str, conn: MainDbConn) -> DRResult<Status>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    conn.run(move |c| {
        diesel::delete(documents.filter(docs::id.eq(real_uuid)).filter(docs::owner.eq(userid.0))).execute(c)
    }).await?;
    Ok(Status::NoContent)
    
}

#[get("/?<name>&<order>&<limit>&<owner>&<offset>&<distinct>&<except>")]
async fn list_docs(userid: UserId, conn: MainDbConn
    , name: Option<String>, order: Option<DocumentOrder>, limit: Option<usize>, owner: bool, offset: Option<i64>
    , distinct: bool, except: Option<String>) -> DRResult<Json<Vec<DocumentInfo>>>{
  
    let vdocs= conn.run(move |c| {
        let mut query = 
            if distinct{
                docs::table.distinct_on(docs::name).into_boxed()
            } else {
                docs::table.into_boxed()
            }
            ;
       
        if let Some (real_name) = name {
            if real_name.contains('*'){
                query = query.filter(docs::name.ilike(real_name.replace("*","%")));
            } else {
                query = query.filter(docs::name.eq(real_name));
            }
        }
        if let Some(Ok(real_except)) = except.map(|s| Uuid::parse_str(&s)) {
            query = query.filter(docs::id.ne(real_except));
        }
        if owner {
            query = query.filter(docs::owner.eq(userid.0));
        }
        let real_order=order.unwrap_or_else(|| DocumentOrder::Recent);
        query =  
            if distinct{
                query.order((docs::name,docs::created.desc()))
            } else {
                match real_order {
                    DocumentOrder::Recent => query.order(docs::created.desc()),
                    DocumentOrder::Name => query.order(docs::name.asc()),
                }
            };

        let real_limit=limit.unwrap_or_else(|| 10);
        let real_offset=offset.unwrap_or_else(|| 0);

        let query = query.select((docs::id,docs::name,docs::created,docs::owner,docs::mime,docs::size));
        
        if distinct {
            let mut query2 = query.sub_select();
            query2 = match real_order {
                DocumentOrder::Recent => query2.order("created desc"),
                DocumentOrder::Name => query2.order("name asc"),
            };
            query2 = query2.limit(real_limit as i64)
                .offset(real_offset);
            //println!("{}",diesel::debug_query(&query2));
            query2.load::<DocumentInfo>(c)
         } else {
            query.limit(real_limit as i64)
                .offset(real_offset)
                .load::<DocumentInfo>(c)
         }

        
        }).await?;
    //println!("length: {}",vdocs.len());
    //println!("vdocs: {:?}",vdocs);
    
    Ok(Json(vdocs))
}

/*
select count(distinct name) from documents

select * from (
select distinct on (name) * from documents order by name, created desc
	) t0 order by created desc
*/

/// count documents
#[get("/count?<name>&<owner>&<distinct>&<except>")]
async fn count_docs(userid: UserId, conn: MainDbConn
    , name: Option<String>, owner: bool, distinct: bool, except: Option<String>) -> DRResult<Json<i64>>{
  
    let cnt= conn.run(move |c| {
        
        let mut query =  if distinct{
            docs::table.distinct().into_boxed()
        } else {
            docs::table.into_boxed()
        };

        if let Some (real_name) = name {
            if real_name.contains('*'){
                query = query.filter(docs::name.ilike(real_name.replace("*","%")));
            } else {
                query = query.filter(docs::name.eq(real_name));
            }
        }
        if let Some(Ok(real_except)) = except.map(|s| Uuid::parse_str(&s)) {
            query = query.filter(docs::id.ne(real_except));
        }
        if owner {
            query = query.filter(docs::owner.eq(userid.0));
        }

        if distinct {
            /*let query2=query.select(docs::name);
            let debug = diesel::debug_query(&query2);
            println!("Debug:{}",debug);
            diesel::sql_query(format!("select count(*) as count from ({}) t0",debug.to_string())).get_result::<GenericCount>(c).map(|gc| gc.count)*/
            query.select(docs::name).count_sub_select().get_result(c)
        } else {
            query.select(diesel::dsl::count(docs::name)).get_result(c)
        }
        
        /*let mut query = String::new();
        if distinct {
            query.push_str("select count(*) as count from ( select name from documents ");
        } else {
            query.push_str("select count(*) as count from documents ");
        }
        if let Some (real_name) = name {
            if real_name.contains('*'){
                query.push_str(" name ilike $1");
                //real_name.replace("*","%")));
            } else {
                query.push_str(" name ilike $1");
            }
        }
        if owner {
            query.push_str("owner=$1");
        }
        
        if distinct {
            query.push_str(") t0");
        }
        diesel::sql_query(query).bind().get_result::<GenericCount>(c).map(|gc| gc.count)*/
        //query
        //    .get_result(c)
        }).await?;
   
    Ok(Json(cnt))
}

#[derive(Debug, PartialEq, FromFormField)]
enum DocumentOrder {
    Recent,
    Name,
}



pub fn routes() -> Vec<Route> {
    routes![upload_doc, get_doc, get_doc_info, get_doc_data, delete_doc, list_docs, count_docs]
}