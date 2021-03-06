use rocket::{Route};
use crate::base::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use crate::model::{Access, User};
use crate::schema::accesses::dsl::accesses;
use crate::schema::users::dsl::users;
use crate::schema::accesses as accs;
use crate::schema::documents::dsl::documents;
use crate::schema::documents as docs;
use crate::schema::users as usrs;

use uuid::Uuid;

use chrono::Utc;
use diesel::prelude::*;


#[get("/<uuid>?<limit>&<offset>")]
async fn get_accesses(_ctx: UserContext,uuid: &str, conn: MainDbConn,limit: Option<usize>, offset: Option<i64>) -> DRResult<Json<Vec<User>>>{
    let real_uuid=Uuid::parse_str(uuid)?;
    let real_limit=limit.unwrap_or(10) as i64;
    let real_offset=offset.unwrap_or(0);

    let usrs:Vec<User>=conn.run(move |c| {
        users.filter(usrs::id.eq_any(accesses.filter(accs::document_id.eq(real_uuid)).select(accs::user_id)))
        .limit(real_limit).offset(real_offset).order(usrs::name)
        .load(c)
    }).await?;
    Ok(Json(usrs))
    
}

#[get("/<uuid>/count")]
async fn count_accesses(_ctx: UserContext,uuid: &str, conn: MainDbConn) -> DRResult<Json<i64>>{
    let real_uuid=Uuid::parse_str(uuid)?;
  
    let usrs:i64=conn.run(move |c| {
        accesses.filter(accs::document_id.eq(real_uuid))
        .count().get_result(c)
    }).await?;
    Ok(Json(usrs))
    
}

async fn has_access(document_id:Uuid, user_id: Uuid, conn: &MainDbConn) -> DRResult<bool> {
    let owner:i64=conn.run(move |c| {
        documents.filter(docs::id.eq(document_id)).filter(docs::owner.eq(user_id)).count().get_result(c)
    }).await?;
    let accs:i64=conn.run(move |c| {
        accesses.filter(accs::document_id.eq(document_id)).filter(accs::user_id.eq(user_id)).count().get_result(c)
    }).await?;
    Ok(owner+accs>0)
}

#[put("/<uuid>/<user>")]
async fn add_access(ctx: UserContext,uuid: &str,user: &str, conn: MainDbConn) -> DRResult<Status>{

    let real_userid=Uuid::parse_str(user)?;
    add_access_internal(ctx,uuid,real_userid,conn).await
}

async fn add_access_internal(ctx: UserContext,uuid: &str,real_userid: Uuid, conn: MainDbConn) -> DRResult<Status>{
    let real_uuid=Uuid::parse_str(uuid)?;
    let accs:i64=conn.run(move |c| {
        accesses.filter(accs::document_id.eq(real_uuid)).filter(accs::user_id.eq(real_userid)).count().get_result(c)
    }).await?;
    if accs==0{
        if !has_access(real_uuid,ctx.user_id,&conn).await?{
            return forbidden();
        }
        let acc=Access{document_id: real_uuid,
            user_id: real_userid,
            created: Utc::now(),};
        conn.run(move |c| {
            diesel::insert_into(accesses)
            .values(&acc)
            .execute(c)
        }).await?;
    }
    Ok(Status::NoContent)
    
}

pub async fn add_access_system(uuid: Uuid,real_userid: Uuid, conn: &MainDbConn) -> DRResult<Status>{
    let accs:i64=conn.run(move |c| {
        accesses.filter(accs::document_id.eq(uuid)).filter(accs::user_id.eq(real_userid)).count().get_result(c)
    }).await?;
    if accs==0{
        let acc=Access{document_id: uuid,
            user_id: real_userid,
            created: Utc::now(),};
        conn.run(move |c| {
            diesel::insert_into(accesses)
            .values(&acc)
            .execute(c)
        }).await?;
    }
    Ok(Status::NoContent)
    
}

#[post("/<uuid>/<email>")]
async fn add_access_email(ctx: UserContext,uuid: &str,email: &str, conn: MainDbConn) -> DRResult<Status>{
    let user_id = ensure_user_exists(email,&conn).await?;
    add_access_internal(ctx,uuid,user_id,conn).await
}

#[delete("/<uuid>/<user>")]
async fn remove_access(ctx: UserContext,uuid: &str,user: &str, conn: MainDbConn) -> DRResult<Status>{
    let real_uuid=Uuid::parse_str(uuid)?;
    let real_userid=Uuid::parse_str(user)?;

    let owner:i64=
        if ctx.user_id == real_userid {
            1
        } else {
            conn.run(move |c| {
            documents.filter(docs::id.eq(real_uuid)).filter(docs::owner.eq(ctx.user_id)).count().get_result(c)
        }).await?
    };

    if owner==1 {
        conn.run(move |c| {
            diesel::delete(accesses.filter(accs::document_id.eq(real_uuid)).filter(accs::user_id.eq(real_userid))).execute(c)
        }).await?;
    } else {
        return forbidden();
    }
    Ok(Status::NoContent)
    
}

pub fn routes() -> Vec<Route> {
    routes![get_accesses,count_accesses,add_access,add_access_email,remove_access]
}