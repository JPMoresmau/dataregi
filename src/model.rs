use uuid::Uuid;
use chrono::{DateTime,Utc};

use crate::schema::*;

#[derive(Queryable, Identifiable, Insertable)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub last_login:DateTime<Utc>,
}

impl User {
    pub fn new<S: Into<String>>(email: S) -> Self {
        User {
            id:Uuid::new_v4(),
            email:email.into(),
            name:Option::None,
            created: Utc::now(),
            last_login: Utc::now(),
        }
    }
}