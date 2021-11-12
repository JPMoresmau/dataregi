use rocket::{Route};
use crate::base::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use crate::model::{Limit, LimitForm};
use crate::schema::limits::dsl::limits;
use crate::schema::limits as lts;

use uuid::Uuid;

use diesel::prelude::*;

#[get("/<user>")]
async fn get_limits(ctx: UserContext, user: &str, conn: MainDbConn) -> DRResult<Json<Limit>>{
    if !ctx.site_admin {
        return forbidden();
    }
    let real_userid=Uuid::parse_str(user)?;
    let lt=conn.run(move |c| {
        limits.filter(lts::user_id.eq(real_userid)).first(c)
    }).await?;
    Ok(Json(lt))
}


#[put("/<user>", data = "<limit>")]
async fn set_limits(ctx: UserContext, user: &str, limit: Json<LimitForm>,conn: MainDbConn) -> DRResult<Status>{
    if !ctx.site_admin {
        return forbidden();
    }
    let real_userid=Uuid::parse_str(user)?;
    conn.run(move |c| {
        diesel::update(limits.filter(lts::user_id.eq(real_userid))).set(&limit.0).execute(c)
    }).await?;

    Ok(Status::NoContent)
}

#[get("/")]
async fn my_limits(ctx: UserContext, conn: MainDbConn) -> DRResult<Json<Limit>>{
    let lt=conn.run(move |c| {
        limits.filter(lts::user_id.eq(ctx.user_id)).first(c)
    }).await?;
    Ok(Json(lt))
}


pub fn routes() -> Vec<Route> {
    routes![get_limits,set_limits,my_limits]
}