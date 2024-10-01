use rocket::{fairing::AdHoc, form::Form, http::{Cookie, CookieJar, Status}, request::{self, FromRequest, Outcome}, serde::Serialize, Request};
use diesel::prelude::*;

use crate::{db::*, schema::*, config::Config};
use rocket_dyn_templates::{context, Template};



#[derive(FromForm)]
struct PasswordLogin {
    email: String,
    password: String,
}

#[derive(Debug, Clone, Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct CurrentUser {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CurrentUser {
    type Error = String;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jar = req.guard::<&CookieJar<'_>>().await.unwrap();
        if let Some(uid_cookie) = jar.get_private("user_id") {
            let uid = uid_cookie.value().parse::<i32>().unwrap();
            let db = req.guard::<Db>().await.unwrap();
            let user: User = db.run(move |conn| {
                users::table
                .filter(users::id.eq(uid))
                .first(conn)
            }).await.unwrap();
            Outcome::Success(CurrentUser {
                id: uid,
                name: user.name,
                email: user.email,
            })
        } else {
            Outcome::Forward(Status::Unauthorized)
        }
    }
}

async fn add_user(db: &Db, name: String, email: String, password: String) -> DbResult<()> {
    #[derive(Debug, Clone, Queryable, Insertable)]
    #[diesel(table_name = users)]
    pub struct NewUser {
        pub name: String,
        pub email: String,
        pub password_hash: Option<String>,
    }
    db.run(move |conn| {
        diesel::insert_into(users::table)
        .values(&NewUser {
            email: email,
            password_hash: Some(pass_hash(password)),
            name: name,
        })
        .execute(conn)
    }).await?;
    Ok(())

}

fn pass_hash(plain: String) -> String {
    use password_hash::{PasswordHasher, 
        rand_core::OsRng, SaltString};
    use argon2::Argon2;
    let salt = SaltString::generate(&mut OsRng);
    
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();
    
    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(plain.as_bytes(), &salt).unwrap().to_string();
    password_hash
}

fn pass_verify(plain: String, hash: String) -> bool {
    use password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;
    // Verify password against PHC string
    let parsed_hash = PasswordHash::new(&hash).unwrap();
    Argon2::default().verify_password(plain.as_bytes(), &parsed_hash).is_ok()
}

#[post("/login", data="<login>")]
async fn login_post(login: Form<PasswordLogin>, jar: &CookieJar<'_>, db: Db) -> DbResult<()>{
    let user_count: i64 = db.run(move |conn| {
        users::table
        .count().get_result(conn)
    }).await?;
    if user_count == 0 {
        add_user(&db, String::from(""), login.email.clone(), login.password.clone()).await?;
    }
    let email = login.email.clone();
    let login_user: User = db.run(move |conn| {
        users::table
        .filter(users::email.eq(email))
        .first(conn)
    }).await?;
    if pass_verify(login.password.clone(), login_user.password_hash.unwrap().clone()) {
        jar.add_private(("user_id", login_user.id.to_string()));
    }
    Ok(())
}

#[get("/login")]
async fn login(config: &Config) -> Template {
    Template::render("login", context! {
        config: config
    })
}

pub fn users_stage() -> AdHoc {
    AdHoc::on_ignite("User Routes", |rocket| async {
        rocket
            .mount("/user", routes![login_post, login])
    })
}