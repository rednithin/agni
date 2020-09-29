use warp::{self, Filter};
use tokio;
use pretty_env_logger;
use std::env;
use std::sync::{Arc,Mutex};
use std::collections::HashMap;

pub mod broadcast;
pub mod handlers;
pub mod types;
pub mod util;

use handlers::{
    root_handler,
    content_desc_handler,
    content_handler
};

use types::AppState;

use util::{get_cache};

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG","info");
    pretty_env_logger::init();
    
    let broadcast_presence = broadcast::get_broadcast_presence_func();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(5000));
        loop {
            interval.tick().await;
            broadcast_presence();
        }
    });

    let mut cache = get_cache();
    let mut item_map = HashMap::new();
    for items in cache.get_mut(&0) {
        for item in items {
            item_map.insert(item.id, item.clone());
        }
    }
    
    let id_counter = (cache.get_mut(&0).unwrap().len() + 1) as u64;
    let app_state = AppState {
        cache,
        id_counter,
        item_map
    };
    let app_state = Arc::new(Mutex::new(app_state));
   
    let routes = warp::any()
        .and(root_handler())
        .or(content_desc_handler())
        .or(content_handler(app_state.clone()))
        .or(
            warp::any()
                .and(warp::fs::dir("/"))
        )
        .with(warp::log::log("agni"));
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
