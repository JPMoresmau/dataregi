use rocket::serde::{Deserialize, Serialize};

use rocket_sync_db_pools::database;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use uuid::Uuid;

#[derive(Serialize)]
pub struct IndexContext<'r> {
    pub error: &'r str,
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
