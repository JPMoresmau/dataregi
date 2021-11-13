use rocket::{Route};
use crate::base::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use crate::model::{Member, MemberInfo, Organization, User};
use crate::schema::members::dsl::members;
use crate::schema::members as mbrs;
use crate::schema::organizations::dsl::organizations;
use crate::schema::organizations as orgs;

use uuid::Uuid;
use chrono::Utc;
use diesel::prelude::*;

#[get("/<org>")]
async fn get_organization(_ctx: UserContext, org: &str, conn: MainDbConn) -> DRResult<Json<Organization>>{
    let org_id=Uuid::parse_str(org)?;
    
    let org=conn.run(move |c| {
        organizations.filter(orgs::id.eq(org_id)).first(c)
    }).await?;
    Ok(Json(org))
}


#[get("/count?<member>")]
async fn get_organization_count(ctx: UserContext, member: bool, conn: MainDbConn) -> DRResult<Json<i64>>{
    
    let cnt=conn.run(move |c| {
        if member {
            organizations
                .filter(orgs::id.eq_any(members.filter(mbrs::user_id.eq(ctx.user_id)).select(mbrs::org_id)))
                .count().get_result(c)
        } else {
            organizations.count().get_result(c)
        }
    }).await?;
    Ok(Json(cnt))
}


#[get("/?<member>&<limit>&<offset>")]
async fn get_organizations(ctx: UserContext, member: bool, limit: Option<usize>, offset: Option<i64>, conn: MainDbConn) -> DRResult<Json<Vec<Organization>>>{
    let real_limit=limit.unwrap_or(10);
    let real_offset=offset.unwrap_or(0);

    let orgs=conn.run(move |c| {
        if member {
            organizations
                .filter(orgs::id.eq_any(members.filter(mbrs::user_id.eq(ctx.user_id)).select(mbrs::org_id)))
                .order(orgs::name)
                .limit(real_limit as i64)
                .offset(real_offset).load(c)
        } else {
            organizations
                .order(orgs::name)
                .limit(real_limit as i64)
                .offset(real_offset).load(c)
        }
    }).await?;
    Ok(Json(orgs))
}


#[post("/<org>")]
async fn set_organization(ctx: UserContext, org: String, conn: MainDbConn) -> DRResult<Json<Organization>>{
    if !ctx.site_admin {
        return forbidden();
    }
    
    let org=conn.run(move |c| {
        let morg=organizations.filter(orgs::name.eq(&org)).first(c).optional()?;
        match morg {
            None => {
                let org=Organization{id:Uuid::new_v4(),name:org,created:Utc::now()};
                diesel::insert_into(organizations)
                    .values(&org)
                    .execute(c)?;
                Ok::<Organization,diesel::result::Error>(org)
                },
            Some(org) => Ok(org),
        }
        
    }).await?;
    Ok(Json(org))
}


#[delete("/<org>")]
async fn delete_organization(ctx: UserContext, org: &str, conn: MainDbConn) -> DRResult<Status>{
    if !ctx.site_admin {
        return forbidden();
    }
    let org_id=Uuid::parse_str(org)?;
    
    conn.run(move |c| {
        diesel::delete(organizations.filter(orgs::id.eq(org_id))).execute(c)
    }).await?;
    Ok(Status::NoContent)
}

fn is_admin(ctx: &UserContext, org_id: Uuid) -> bool {
    ctx.org_members.iter().any(|m| m.org_id==org_id && m.org_admin)
}

#[get("/<org>/members?<limit>&<offset>")]
async fn get_members(ctx: UserContext, org: &str, limit: Option<usize>, offset: Option<i64>, conn: MainDbConn) -> DRResult<Json<Vec<MemberInfo>>>{
    let org_id=Uuid::parse_str(org)?;
    if !(ctx.site_admin || is_admin(&ctx,org_id)) {
        return forbidden();
    }
    let real_limit=limit.unwrap_or(10);
    let real_offset=offset.unwrap_or(0);

    use crate::schema::users::dsl::*;
    use crate::schema::users as usrs;
    let mbrs=conn.run(move |c| {
        members.filter(mbrs::org_id.eq(org_id))
            .inner_join(users)
            .select((mbrs::user_id,usrs::email,usrs::name,mbrs::org_admin))
            .order(usrs::name)
            .limit(real_limit as i64)
            .offset(real_offset).load::<MemberInfo>(c)
    }).await?;
    Ok(Json(mbrs))
}

#[get("/<org>/members/count")]
async fn get_members_count(ctx: UserContext, org: &str, conn: MainDbConn) -> DRResult<Json<i64>>{
    let org_id=Uuid::parse_str(org)?;
    if !(ctx.site_admin || is_admin(&ctx,org_id)) {
        return forbidden();
    }

    let cnt=conn.run(move |c| {
        members.filter(mbrs::org_id.eq(org_id))
            .count().get_result(c)
    }).await?;
    Ok(Json(cnt))
}


#[get("/<org>/<user>")]
async fn get_member(ctx: UserContext, org: &str, user: &str, conn: MainDbConn) -> DRResult<Json<Option<Member>>>{
    let org_id=Uuid::parse_str(org)?;
    if !(ctx.site_admin || is_admin(&ctx,org_id)) {
        return forbidden();
    }
    
    let user_id=Uuid::parse_str(user)?;

    let mbr=conn.run(move |c| {
        members.filter(mbrs::user_id.eq(user_id).and(mbrs::org_id.eq(org_id))).first(c).optional()
    }).await?;
    Ok(Json(mbr))
}

#[put("/<org>/<user>?<admin>")]
async fn set_member(ctx: UserContext, org: &str, user: &str, admin:bool, conn: MainDbConn) -> DRResult<Json<Member>>{
    let org_id=Uuid::parse_str(org)?;
    if !(ctx.site_admin || is_admin(&ctx,org_id)) {
        return forbidden();
    }
    
    let user_id=match Uuid::parse_str(user){
        Ok(uuid)=> uuid,
        Err(_)=> {
            use crate::schema::users::dsl::*;
            use crate::schema::limits::user_id;
            use crate::schema::limits::dsl::limits as lts;
            let mail=String::from(user);
            let ouser = conn
                .run(move |c| users.filter(email.eq(mail)).first::<User>(c).optional())
                .await?;
            match ouser {
                Some(user)=>user.id,
                None=>{
                    let user = User::new_login(user);
                    conn.run(move |c| {
                        let ctx = diesel::insert_into(users)
                            .values(&user)
                            .execute(c)
                            .map(|_| user.id);
                        diesel::insert_into(lts)
                            .values(user_id.eq(user.id))
                            .execute(c)?;
                        ctx
                    })
                    .await?
                },
            }
        },
    };

    let mbr=conn.run(move |c| {
        let ombr: Option<Member> = members.filter(mbrs::user_id.eq(user_id).and(mbrs::org_id.eq(org_id))).first(c).optional()?;
        match ombr {
            None => {
                let mbr=Member{user_id,org_id,created:Utc::now(),org_admin:admin};
                diesel::insert_into(members).values(&mbr).execute(c)?;
                Ok::<Member,diesel::result::Error>(mbr)
            },
            Some(mut mbr) => {
                mbr.org_admin=admin;
                diesel::update(&mbr).set(mbrs::org_admin.eq(admin)).execute(c)?;
                Ok(mbr)
            },
        }
    }).await?;
    Ok(Json(mbr))
}

#[delete("/<org>/<user>")]
async fn delete_member(ctx: UserContext, org: &str, user: &str,conn: MainDbConn) -> DRResult<Status>{
    let org_id=Uuid::parse_str(org)?;
    if !(ctx.site_admin || is_admin(&ctx,org_id)) {
        return forbidden();
    }
    
    let user_id=Uuid::parse_str(user)?;

   conn.run(move |c| {
        diesel::delete(members.filter(mbrs::user_id.eq(user_id).and(mbrs::org_id.eq(org_id)))).execute(c)
    }).await?;
    Ok(Status::NoContent)
}

pub fn routes() -> Vec<Route> {
    routes![get_organization, get_organization_count,get_organizations, set_organization,delete_organization
        ,get_members,get_members_count, get_member, set_member,delete_member]
}