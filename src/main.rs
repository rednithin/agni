use actix_web::{App, HttpServer,};
use tokio;
use pretty_env_logger;
use std::env;
use std::sync::{Arc,Mutex};
use std::collections::HashMap;
use uuid::{Uuid};

pub mod broadcast;
pub mod handlers;
pub mod types;
pub mod util;

use handlers::config;

use types::AppState;

use util::get_cache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG","info");
    pretty_env_logger::init();

    let uuid = Uuid::new_v4();
    
    let broadcast_presence = broadcast::get_broadcast_presence_func(uuid.clone(), None);
    let handle1 = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(2000));
        loop {
            interval.tick().await;
            broadcast_presence();
        }
    });

    let handle2 = tokio::task::spawn_blocking(move || {
        broadcast::listen_to_discover_messages(uuid.clone());
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
        item_map,
        uuid,
    };
    let app_state = Arc::new(Mutex::new(app_state));
   
    let handle3 = HttpServer::new(move || {
            App::new()
                .data(app_state.clone())
                .configure(config)
        })
        .bind("0.0.0.0:3030")?
        .run();
    
    tokio::select! {
        _ = handle1 => 0,
        _ = handle2 => 0,
        _ = handle3 => 0,
    };

    Ok(())
}
