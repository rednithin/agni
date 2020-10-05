use pnet::datalink;
use lru_cache::LruCache;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use crate::types::{ListItemWrapper, ListItem, Item, Container, Res};

const FRAGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'[')
    .add(b']');


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
                    let file_name = entry.file_name().into_string().unwrap();
                    let file_path = entry.path().to_str().unwrap().to_string();
                    let ip = get_local_ip();
                    let file_path =utf8_percent_encode(&file_path, FRAGMENT).to_string();
                    
                    if file_name.ends_with(".mp4") || file_name.ends_with(".mkv") {
                        list_items.push(ListItemWrapper {
                            list_item: ListItem::Item(Item {
                                id: id_counter,
                                parent_id: parent_id,
                                title: entry.file_name().into_string().unwrap(),
                                class: "object.item.videoItem".to_string(),
                                res: Res {
                                    protocol_info: "http-get:*:video/x-matroska:*".to_string(),
                                    content: format!("http://{}:3030/agni-files/{}", ip, file_path)
                                }
                            }),
                            id: id_counter,
                            dir: None,
                        })
                    }
                }
            }
        } else {
            break;
        }
        id_counter += 1;
    }

    list_items.sort_by(|a, b| {
        let compute_string = |list_item_wrapper: &ListItemWrapper| {
            match list_item_wrapper.list_item.clone() {
                ListItem::Container(x) => format!("D{}", x.title),
                ListItem::Item(x) => format!("F{}", x.title),
            }
        };
        compute_string(a).cmp(&compute_string(b))
    });
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