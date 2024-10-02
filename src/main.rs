#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_sync_db_pools;

use std::{fmt::Debug, path::PathBuf, sync::atomic::{AtomicI64, Ordering}};

use reqwest::dns::Name;
use rocket::{fairing::AdHoc, http::hyper::Uri};
/*use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx::{self, Row};*/

//use rocket::serde::json::Json;
use rocket_dyn_templates::{context, Template};
use rocket::{fairing, Build, Rocket, State, fs::NamedFile};

use diesel::prelude::*;
mod schema;
mod db;
mod user;
mod rss_gen;
mod config;
mod post;
mod time_util;

use config::*;
use schema::*;
use db::*;

use rocket::serde::{Serialize, Deserialize, json::Json};
use user::{users_stage, CurrentUser};

struct AppState {
    count: AtomicI64
}



fn sanitize_path(base: PathBuf, path: PathBuf) -> Option<PathBuf> {
    let prefix = if base.is_absolute() {
        base.canonicalize().ok()?
    } else {
        let mut cwd = std::env::current_dir().ok()?;
        cwd.push(base);
        cwd.canonicalize().ok()?
    };
    let mut output = prefix.clone();
    output.push(path);
    let mut output = output.canonicalize().ok()?;
    if output.starts_with(prefix) {
        Some(output)
    } else {
        None
    }
}

#[get("/static/<path..>")]
async fn static_file(path: PathBuf) -> Option<NamedFile> {
    if let Some(file_path) = sanitize_path("static".parse::<PathBuf>().unwrap(), path) {
        NamedFile::open(file_path).await.ok()
    } else {
        None
    }

}


#[get("/")]
async fn index(state: &State<AppState>, db:Db, user: Option<CurrentUser>, config: &State<Config>) -> DbResult<Template> {
    let hits = state.count.fetch_add(1, Ordering::Relaxed);
    let post_list = db.get_all_posts().await?;
    Ok(Template::render("index", context! {
        config: config.inner(),
        user: user,
        count: hits,
        posts: post_list,
    }))
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    Db::get_one(&rocket).await
        .expect("database connection")
        .run(|conn| { conn.run_pending_migrations(MIGRATIONS).expect("diesel migrations"); })
        .await;

    rocket
    /*
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        }
        None => Err(rocket),
    }*/
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(AppState { count: AtomicI64::new(0)})
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
        .attach(Template::fairing())
        .attach(users_stage())
        .attach(config_stage())
        .attach(rss_gen::rss_stage())
        .attach(post::posts_stage())
        .mount("/", routes![index, static_file])
}
