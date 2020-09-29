use pnet::datalink;
use lru_cache::LruCache;
use crate::types::{ListItemWrapper, ListItem, Container};
use warp::Filter;

pub fn get_local_ip() -> std::net::IpAddr {
    let interfaces = datalink::interfaces();
    let location = interfaces
        .iter()
        .find(|&x| {
            // let s = format!("{}", x);
            // s.contains("192.168")
            x.is_broadcast() && x.is_multicast()
        }
    ).unwrap();
    
    location.ips[0].ip()
}

pub struct ReadDirectoryReturnType {
    pub list_items:Vec<ListItemWrapper>,
    pub id_counter: u64
}

pub async fn read_directory(path: String, parent_id: u64, mut id_counter: u64) -> ReadDirectoryReturnType {
    let mut entries = tokio::fs::read_dir(&path).await.unwrap();

    let mut list_items = vec![];

    while let Ok(entry) = entries.next_entry().await {
        if let Some(entry) = entry {
            if let Ok(entry_type) = entry.file_type().await {
                if entry_type.is_dir() {
                    list_items.push(ListItemWrapper {
                        list_item: ListItem::Container(Container {
                            id: id_counter,
                            parent_id: parent_id,
                            title: entry.file_name().into_string().unwrap(),
                            class: "object.container.storageFolder".to_string(),
                        }),
                        id: id_counter,
                        dir: Some(entry.path().to_str().unwrap().to_string())
                    })
                } else {
                    // files.push((entry.file_name().into_string().unwrap(), {
                    //     let x = entry.path();
                    //     x.to_str().unwrap()[1..].to_string()
                    // }));
                }
            }
        } else {
            break;
        }
        id_counter += 1;
    }

    ReadDirectoryReturnType {
        list_items,
        id_counter
    }
}

pub fn get_cache() -> LruCache<u64,Vec<ListItemWrapper>> {

    let mut initial_list_items = vec![ListItemWrapper {
        list_item: ListItem::Container(Container {
            id: 1,
            parent_id: 0,
            title: "Documents".to_string(),
            class: "object.container.storageFolder".to_string(),
        }),
        id: 1,
        dir: Some("/home/nithin/Server".into())
    }];

    for (i, x) in initial_list_items.iter_mut().enumerate() {
        let id = i as u64 +1;
        x.id = id;
        match &mut x.list_item {
            ListItem::Container(x) => x.id = id,
            ListItem::Item(x) => x.id = id,
        }
    }

    let mut cache: LruCache<u64, Vec<ListItemWrapper>> = LruCache::new(100);

    cache.insert(0, initial_list_items);

    cache
}

pub fn with_cloneable<T: Clone + std::marker::Send>(
    t: T,
) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || t.clone())
}