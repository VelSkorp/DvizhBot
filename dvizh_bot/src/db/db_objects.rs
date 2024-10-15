use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub first_name: Option<String>,
    pub birthdate: Option<String>,
    pub language_code: Option<String>
}

impl User {
    pub fn new(username: String, first_name: Option<String>, birthdate: Option<String>, language_code: Option<String>) -> Self {
        User {
            username,
            first_name,
            birthdate,
            language_code
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub id: i64,
    pub title: Option<String>
}

impl Chat {
    pub fn new(id: i64, title: Option<String>) -> Self {
        Chat {
            id,
            title
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Members {
    pub group_id: i64,
    pub user_id: i64,
}