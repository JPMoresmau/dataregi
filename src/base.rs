use std::io::Cursor;
use rocket::serde::{Deserialize, Serialize};

use std::io::Error as IOError;
use diesel::result::Error as DieselError;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::sql_types::BigInt;

use rocket::http::{ContentType,Status};
use rocket::request::Request;
use rocket::response::{Responder, Result, Response};
use rocket_sync_db_pools::database;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use figment::value::magic::RelativePathBuf;
use uuid::Uuid;
use std::fmt;
use std::error::Error as StdError;
use rocket::serde::json::Json;
use crate::model::Member;

pub const COOKIE: &str = "user";

#[derive(Serialize,Debug)]
pub struct IndexContext<'r> {
    pub error: &'r str,
    pub message: &'r str,
    pub callback_name: &'r str,
}

#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct UserContext {
    pub user_id: Uuid,
    pub site_admin: bool,
    pub org_members: Vec<Membership>,
}

impl UserContext {
    pub fn new(user_id: Uuid, site_admin: bool) -> Self {
        UserContext{user_id, site_admin,org_members:vec![]}
    }

    pub fn new_in_org(user_id: Uuid,members: &[Member], site_admin: bool) -> Self {
        UserContext{user_id, site_admin,org_members:members.iter().map(|m| Membership{org_id:m.org_id,org_admin:m.org_admin}).collect()}
    }
}

#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct Membership {
    pub org_id: Uuid,
    pub org_admin: bool,
}

#[derive(Serialize,Debug)]
pub struct DocumentContext<'r> {
    pub user_id: &'r Uuid,
    pub doc_id: &'r str,
}

#[derive(Serialize,Debug)]
pub struct OrganizationContext<'r> {
    pub user_id: &'r Uuid,
    pub org_id: &'r str,
}


#[derive(Deserialize)]
pub struct LoginEmail<'r> {
    pub address: &'r str,
}

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
    pub callback_name: String,
    pub token_lifespan_minutes: u64,
    pub temp_dir: RelativePathBuf,
}

pub struct LoginRegistration {
    pub address: String,
    pub timestamp: Instant,
}

impl LoginRegistration {
    pub fn new<S: Into<String>>(address: S) -> Self {
        LoginRegistration {
            address: address.into(),
            timestamp: Instant::now(),
        }
    }
}

pub struct EmailTokens {
    pub tokens: Mutex<HashMap<String, LoginRegistration>>,
}

impl Default for EmailTokens {
    fn default() -> Self {
        EmailTokens {
            tokens: Mutex::new(HashMap::new()),
        }
    }
}

#[database("postgres_main")]
pub struct MainDbConn(diesel::PgConnection);

#[derive(Serialize, Debug)]
pub struct StructuredError{
    #[serde(skip_serializing)]
    status: Status,
    error_type: DRError,
    error: String,
}

impl StructuredError {
    pub fn not_found<T:Into<String>>(msg: T) -> Self{
        StructuredError{status:Status::NotFound,
            error_type:DRError::NotFoundError,
            error:msg.into()}
    }
}

impl<'r,'o: 'r> Responder<'r,'o> for StructuredError {
    fn respond_to(self, request: &'r Request<'_>) -> Result<'o> {
        println!("error: {}",self);
        
        Response::build_from(Json(&self).respond_to(request).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
        
    }
}

pub struct Download{
    pub content_type: ContentType,
    pub filename: String,
    pub data: Vec<u8>,
}

impl<'r,'o: 'r> Responder<'r,'o> for Download {
    fn respond_to(self, _request: &'r Request<'_>) -> Result<'o> {
        Response::build().sized_body(self.data.len(),Cursor::new(self.data))
            .header(self.content_type)
            .raw_header("Content-Disposition",format!("attachment; filename=\"{}\"",self.filename))
            .ok()
    }
}

#[derive(Serialize, Debug)]
pub enum DRError {
    DatabaseError,
    IOError,
    UuidError,
    NotFoundError,
    ForbiddenError,
}

impl fmt::Display for StructuredError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.error)
    }
}

impl StdError for StructuredError {
    fn description(&self) -> &str {
        &self.error
    }
}

pub fn forbidden<T>() -> DRResult<T>{
    Err(StructuredError{status:Status::Forbidden,
        error_type:DRError::ForbiddenError,
        error: String::new()})
}

impl From<DieselError> for StructuredError {
    fn from(e: DieselError) -> Self {
        StructuredError{status:Status::InternalServerError,
            error_type:DRError::DatabaseError,
            error: e.to_string()}
    }
}

impl From<IOError> for StructuredError {
    fn from(e: IOError) -> Self {
        StructuredError{status:Status::InternalServerError,
            error_type:DRError::IOError,
            error:e.to_string()}
    }
}

impl From<uuid::Error> for StructuredError {
    fn from(e: uuid::Error) -> Self {
        StructuredError{status:Status::BadRequest,
            error_type:DRError::UuidError,
            error:e.to_string()}
    }
}

pub type DRResult<T> = std::result::Result<T, StructuredError>;


#[derive(Debug, Clone, Copy, QueryId)]
pub struct CountedSubSelect<T> {
    query: T
}

impl<T: Query> Query for CountedSubSelect<T> {
    type SqlType = BigInt;
}

impl<T> RunQueryDsl<PgConnection> for CountedSubSelect<T> {}

impl<T> QueryFragment<Pg> for CountedSubSelect<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT COUNT(*) FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t");
        Ok(())
    }
}

pub trait CountSubSelect: Sized {
    fn count_sub_select(self) -> CountedSubSelect<Self>;
}

impl<T> CountSubSelect for T {
    fn count_sub_select(self) -> CountedSubSelect<Self> {
        CountedSubSelect {
            query: self,
        }
    }
}

#[derive(Debug, Clone, QueryId)]
pub struct SelectedSubSelect<T> {
    query: T,
    limit: i64,
    offset: i64,
    order: String,
}

impl <T> SelectedSubSelect<T> {
    pub fn limit(self,limit: i64) -> Self {
        SelectedSubSelect{
            query: self.query,
            limit,
            offset: self.offset,
            order: self.order,
        }
    }

    pub fn offset(self,offset: i64) -> Self {
        SelectedSubSelect{
            query: self.query,
            limit: self.limit,
            offset,
            order: self.order,
        }
    }

    pub fn order<S: Into<String>>(self, e:S) -> Self {
        SelectedSubSelect{
            query: self.query,
            limit: self.limit,
            offset: self.offset,
            order: e.into(),
        }
    }
}

impl<T: Query> Query for SelectedSubSelect<T> {
    type SqlType = T::SqlType;
}



impl<T> RunQueryDsl<PgConnection> for SelectedSubSelect<T> {}

impl<T> QueryFragment<Pg> for SelectedSubSelect<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT * FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t");
        if !self.order.is_empty(){
            out.push_sql(&format!(" ORDER BY {}",self.order))
        }
        if self.limit>0{
            out.push_sql(&format!(" LIMIT {}",self.limit))
        }
        if self.offset>0{
            out.push_sql(&format!(" OFFSET {}",self.offset))
        }

        Ok(())
    }
}

pub trait SelectSubSelect: Sized {
    fn sub_select(self) -> SelectedSubSelect<Self>;
}

impl<T> SelectSubSelect for T {
    fn sub_select(self) -> SelectedSubSelect<Self> {
        SelectedSubSelect {
            query: self,
            limit: 0,
            offset: 0,
            order: String::new()
        }
    }
}

pub async fn ensure_user_exists(user_email: &str, conn: &MainDbConn) -> DRResult<Uuid> {
    use crate::model::User;
    use crate::schema::users::dsl::*;
    use crate::schema::limits::user_id;
    use crate::schema::limits::dsl::limits as lts;
    let mail=String::from(user_email);
    let ouser = conn
        .run(move |c| users.filter(email.eq(mail)).first::<User>(c).optional())
        .await?;
    match ouser {
        Some(user)=>Ok(user.id),
        None=>{
            let user = User::new_reference(user_email);
            let new_id = conn.run(move |c| {
                let ctx = diesel::insert_into(users)
                    .values(&user)
                    .execute(c)
                    .map(|_| user.id);
                diesel::insert_into(lts)
                    .values(user_id.eq(user.id))
                    .execute(c)?;
                ctx
            }).await?;
            Ok(new_id)
        },
    }
}