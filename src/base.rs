use rocket::serde::{Deserialize, Serialize};

use std::io::Error as IOError;
use diesel::result::Error as DieselError;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::sql_types::BigInt;

use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder,Result};
use rocket_sync_db_pools::database;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use figment::value::magic::RelativePathBuf;
use uuid::Uuid;
use std::fmt;
use std::error::Error as StdError;

#[derive(Serialize)]
pub struct IndexContext<'r> {
    pub error: &'r str,
    pub message: &'r str,
}

#[derive(Serialize)]
pub struct UserContext<'r> {
    pub user_id: &'r Uuid,
}

pub struct UserId(pub Uuid);

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


#[derive(Debug)]
pub enum DRError{
    DatabaseError(String),
    IOError(String),
    UuidError(String),
    NotFoundError,
}

impl<'r,'o: 'r> Responder<'r,'o> for DRError {
    fn respond_to(self, _request: &'r Request<'_>) -> Result<'o> {
        println!("error: {}",self);
        match self {
            DRError::NotFoundError=> Err(Status::NotFound),
            _ => Err(Status::InternalServerError),
        }
        
    }
}

impl fmt::Display for DRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            DRError::DatabaseError(msg)=>msg,
            DRError::IOError(msg)=>msg,
            DRError::UuidError(msg)=>msg,
            DRError::NotFoundError=> "Not Found",
        })
    }
}

impl StdError for DRError {
    fn description(&self) -> &str {
        match self {
            DRError::DatabaseError(msg)=>msg,
            DRError::IOError(msg)=>msg,
            DRError::UuidError(msg)=>msg,
            DRError::NotFoundError=> "Not Found",
        }

    }
}

impl From<DieselError> for DRError {
    fn from(e: DieselError) -> Self {
        DRError::DatabaseError(e.to_string())
    }
}

impl From<IOError> for DRError {
    fn from(e: IOError) -> Self {
        DRError::IOError(e.to_string())
    }
}

impl From<uuid::Error> for DRError {
    fn from(e: uuid::Error) -> Self {
        DRError::UuidError(e.to_string())
    }
}

pub type DRResult<T> = std::result::Result<T, DRError>;


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


