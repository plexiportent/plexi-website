use std::{borrow::BorrowMut, ops::DerefMut, sync::atomic::{AtomicBool, Ordering}};

use rocket::{fairing::AdHoc, request::{FromRequest, Outcome}, Request, State};
use rss::{Channel, ChannelBuilder};
use crate::config::Config;
use futures::lock::Mutex;

struct RssCtx {
    initialized: AtomicBool,
    latest: Mutex<String>,
}

impl RssCtx {
    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }
    async fn get_latest(&self) -> String {
        self.latest.lock().await.clone()
    }
    async fn set_latest(&self, new_latest: String) {
        let mut lock = self.latest.lock().await;
        let guarded: &mut String = lock.deref_mut();
        *guarded = new_latest;
        self.initialized.store(true, Ordering::Relaxed);
    }
    fn new() -> Self {
        RssCtx {
            initialized: false.into(),
            latest: Mutex::new(String::from("")),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r RssCtx {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let rss_ctx = req.guard::<&State<RssCtx>>().await.map(|thing| thing.inner());
        match rss_ctx {
            Outcome::Success(ctx) => {
                if !ctx.is_initialized() {
                    regen_rss_from_request(req, ctx).await;
                }
                Outcome::Success(ctx)
            },
            x => x
        }
    }
}

async fn regen_rss_from_request<'r>(req: &'r Request<'_>, rss_ctx: &RssCtx) -> String {
    let config = req.guard::<&Config>().await.unwrap();

    let channel = ChannelBuilder::default()
        .title(&config.title)
        .link(&config.base_uri)
        .description(&config.description)
        .build();
    let mut buf: Vec<u8> = Vec::new();
    channel.pretty_write_to(&mut buf, b' ', 2);
    let written = String::from_utf8(buf).unwrap();
    rss_ctx.set_latest(written.clone()).await;
    written
}


#[get("/")]
async fn rss_index(rss_ctx: &RssCtx) -> String {
    rss_ctx.get_latest().await
}

pub fn rss_stage() -> AdHoc {
    AdHoc::on_ignite("Rss Routes", |rocket| async {
        rocket
            .manage(RssCtx::new())
            .mount("/rss", routes![rss_index])
    })
}