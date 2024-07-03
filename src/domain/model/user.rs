use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(email: &str, password: &str) -> User {
        let now = Utc::now();
        User {
            id: uuid::Uuid::new_v4(),
            email: email.to_string(),
            password: password.to_string(),
            created_at: Some(now),
            updated_at: Some(now),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FilteredUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<&User> for FilteredUser {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            email: user.email.to_string(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
