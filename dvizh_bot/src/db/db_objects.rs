use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub first_name: Option<String>,
    pub birthdate: Option<String>,
    pub language_code: Option<String>,
}

impl User {
    pub fn new(
        username: String,
        first_name: Option<String>,
        birthdate: Option<String>,
        language_code: Option<String>,
    ) -> Self {
        User {
            username,
            first_name,
            birthdate,
            language_code,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub id: i64,
    pub title: String,
    pub language_code: String,
}

impl Chat {
    pub fn new(id: i64, title: String, language_code: String) -> Self {
        Chat {
            id,
            title,
            language_code,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Members {
    pub group_id: i64,
    pub user_id: i64,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub group_id: i64,
    pub title: String,
    pub location: String,
    pub date: String,
    pub description: String,
}

impl Event {
    pub fn new(
        group_id: i64,
        title: String,
        date: String,
        location: String,
        description: String,
    ) -> Self {
        Event {
            group_id,
            title,
            location,
            date,
            description,
        }
    }
}
