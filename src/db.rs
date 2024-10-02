
use rocket::serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::*;


//#[derive(Database)]
#[database("sqlite_db")]
pub struct Db(diesel::SqliteConnection);

impl Db {
    pub async fn get_all_posts(&self) -> DbResult<Vec<Post>> {
        let mut post_list: Vec<Post> = self.run(move |conn| {
            posts::table
            .load(conn)

        }).await?;
        post_list.sort_by_key(|p| p.timestamp);
        post_list.reverse();
        Ok(post_list)
    }

}

pub type DbResult<T, E=rocket::response::Debug<diesel::result::Error>> = std::result::Result<T, E>;


#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub author: i32,
    pub timestamp: i64,
}


#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password_hash: Option<String>,
}