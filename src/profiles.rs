use rocket::{Route};
use crate::base::*;
use rocket::serde::json::Json;
use crate::model::{User};
use crate::schema::users::dsl::users;
use crate::schema::users as usrs;

use diesel::prelude::*;


#[get("/")]
async fn get_profile(ctx: UserContext, conn: MainDbConn) -> DRResult<Json<User>>{
    let u=conn.run(move |c| {
        users.filter(usrs::id.eq(ctx.user_id)).first(c)
    }).await?;
    Ok(Json(u))
}

pub fn routes() -> Vec<Route> {
    routes![get_profile]
}