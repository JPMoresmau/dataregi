use rocket::serde::{Deserialize, Serialize};

use std::io::Error as IOError;
use diesel::result::Error as DieselError;
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