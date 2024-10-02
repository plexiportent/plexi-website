use rocket::{fairing::AdHoc, form::Form, http::{Cookie, CookieJar, Status}, request::{self, FromRequest, Outcome}, serde::Serialize, Request};
use diesel::prelude::*;

use crate::{db::*, schema::*, user::CurrentUser, config::Config};
use rocket_dyn_templates::{context, Template};



#[derive(FromForm)]
struct PostForm {
    title: String,
    content: String,
}

async fn create_new_post(db: &Db, title: String, content: String, user_id: i32)  -> DbResult<()> {
    #[derive(Debug, Clone, Serialize, Queryable, Insertable)]
    #[serde(crate = "rocket::serde")]
    #[diesel(table_name = posts)]
    pub struct NewPost {
        pub title: String,
        pub content: String,
        pub author: i32,
        pub timestamp: i64,
    }
    db.run(move |conn| {
        diesel::insert_into(posts::table)
        .values(&NewPost {
            title: title,
            content: content,
            author: user_id,
            timestamp: crate::time_util::current_unix_timestamp(),
        })
        .execute(conn)
    }).await?;
    Ok(())
}


#[get("/new")]
async fn new_post(user: CurrentUser, config: &Config) -> Template {
    Template::render("new_post", context! {
        user: user,
        config: config
    })
}

#[post("/", data="<post_form>")]
async fn post_post(db: Db, post_form: Form<PostForm>, user: CurrentUser) -> DbResult<()> {
    create_new_post(&db, post_form.title.clone(), post_form.content.clone(), user.id).await?;
    Ok(())
}

pub fn posts_stage() -> AdHoc {
    AdHoc::on_ignite("Post Routes", |rocket| async {
        rocket
            .mount("/post", routes![post_post, new_post])
    })
}