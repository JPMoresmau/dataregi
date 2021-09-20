use uuid::Uuid;
use chrono::{DateTime,Utc};
use rocket::serde::{Deserialize, Serialize};
use crate::schema::*;

#[derive(Queryable, Identifiable, Insertable, Deserialize, Serialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl User {
    pub fn new_login<S: Into<String>>(email: S) -> Self {
        User {
            id:Uuid::new_v4(),
            email:email.into(),
            name:Option::None,
            created: Utc::now(),
            last_login: Some(Utc::now()),
        }
    }

    pub fn new_reference<S: Into<String>>(email: S) -> Self {
        User {
            id:Uuid::new_v4(),
            email:email.into(),
            name:Option::None,
            created: Utc::now(),
            last_login: None,
        }
    }
}

#[derive(Queryable, Identifiable, Insertable, Deserialize, Serialize, Debug)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub created: DateTime<Utc>,
    pub owner: Uuid,
    pub mime: Option<String>,
    pub size: i64,
    pub data: Vec<u8>,
    pub hash: Option<String>
}

#[derive(Queryable, Deserialize, Serialize, Debug)]
pub struct DocumentInfo {
    pub id: Uuid,
    pub name: String,
    pub created: DateTime<Utc>,
    pub owner: Uuid,
    pub mime: Option<String>,
    pub size: i64,
}