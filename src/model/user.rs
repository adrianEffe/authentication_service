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

#[deprecated]
#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub user: FilteredUser,
}
