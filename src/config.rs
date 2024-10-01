

use rocket::{fairing::AdHoc, request::{FromRequest, Outcome}, serde::{json::Json, Deserialize, Serialize}, Request, State};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub rss_uri: String,
    pub base_uri: String,
    pub title: String,
    pub description: String,
    
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Config {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        req.guard::<&State<Config>>().await.map(|conf| conf.inner())
    }
}

pub fn config_stage() -> AdHoc {
    AdHoc::on_ignite("Custom Configuration", |rocket| async {
        let figment = rocket.figment();
        let config: Config = figment.extract().expect("config");
        rocket
            .manage(config)
    })
}