use warp::{self, Filter};
use tokio;
use pretty_env_logger;
use std::env;

pub mod broadcast;
pub mod handlers;
pub mod types;
pub mod util;

use handlers::{
    root_handler,
    content_desc_handler,
    content_handler
};

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG","info");
    pretty_env_logger::init();
    
    let broadcast_presence = broadcast::get_broadcast_presence_func();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            broadcast_presence();
        }
    });
   
    let routes = root_handler()
        .or(content_desc_handler())
        .or(content_handler())
        .with(warp::log::log("agni"));
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
