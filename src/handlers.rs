use warp::{Filter, filters::BoxedFilter, Reply};
use bytes::Bytes;
use log;
use uuid::Uuid;
use std::sync::{Arc,Mutex};
use crate::types::{
    Envelope,
    Body,
    BrowseResponse,
    DidlLite,
    ListItemWrapper,
    AppState,
    XMLNS_DC,
    XMLNS_DIDL,
    XMLNS_UPNP,
    ENVELOPE_ENCODING_STYLE,
    XMLNS_ENVELOPE,
    XMLNS_CONTENT_DIRECTORY,
};
use crate::util::{with_cloneable,read_directory};
    

const ROOT_XML: &str = include_str!("root.xml");
const CONTENT_DESC_XML: &str = include_str!("content_desc.xml");
const SOAP_ACTION :&str = "Soapaction";

fn get_browse_response(list_items: &Vec<ListItemWrapper>) -> String {
    let didl_result = DidlLite {
        xmlns_dc: XMLNS_DC.to_string(),
        xmlns_upnp: XMLNS_UPNP.to_string(),
        xmlns: XMLNS_DIDL.to_string(),
        list_items: list_items.iter().map(|x| x.list_item.clone()).collect()
    };
    let response = Envelope {
        encoding_style: ENVELOPE_ENCODING_STYLE.to_string(),
        xmlns: XMLNS_ENVELOPE.to_string(),
        body: Body {
            xmlns: XMLNS_CONTENT_DIRECTORY.to_string(),
            browse_response: BrowseResponse {
                number_returned: 1,
                total_matches: 1,
                update_id: 1,
                result: "{didl-result}".to_string(),
            }
        }
    };
    use strong_xml::{XmlWrite};
    
    let didl_result = didl_result
        .to_string()
        .unwrap()
        .replace('<',"&lt;")
        .replace('>',"&gt;")
        .replace('"',"&quot;");
    
    response
        .to_string()
        .unwrap()
        .replace("{didl-result}", &didl_result)
}

pub fn root_handler(uuid: Uuid) -> BoxedFilter<(impl Reply,)> {
    let uuid_string = uuid.to_string();
    warp::any()
        .and(warp::get())
        .and(warp::path!("root.xml"))
        .map(move || {
            ROOT_XML
                .replace("{name}", "Rednithin")
                .replace("{uuid}", &uuid_string.clone())
        })
        .with(warp::reply::with::header("Content-type", "text/xml"))
        .boxed()
}

pub fn content_desc_handler() -> BoxedFilter<(impl Reply,)> {
    warp::any()
        .and(warp::get())
        .and(warp::path!("content" / "desc.xml"))
        .map(|| CONTENT_DESC_XML)
        .boxed()
}

pub fn content_handler(app_state: Arc<Mutex<AppState>>) -> BoxedFilter<(impl Reply,)> {
    warp::any()
        .and(warp::post())
        .and(warp::path!("content" / "control"))
        .and(with_cloneable(app_state))
        .and(warp::header::<String>(SOAP_ACTION))
        .and(warp::body::bytes())
        .and_then(|app_state: Arc<Mutex<AppState>>, soap_action_header: String, body: Bytes| async move {
            let action = match soap_action_header.trim_matches('"').split("#").collect::<Vec<&str>>().get(1) {
                Some(&x) => x,
                _ => "Error"
            };

            let body_vec = body.to_vec();
            let body_string = String::from_utf8_lossy(&body_vec);

            let xml_doc = roxmltree::Document::parse(&body_string).unwrap();
            let elem = xml_doc.descendants().find(|x| x.tag_name().name() == "ObjectID").unwrap();
            let object_id = if let Ok(x) = elem.text().unwrap().to_string().parse::<u64>() {
                x
            } else {
                std::u64::MAX
            };

            if object_id == std::u64::MAX {
                return Err(warp::reject::not_found());
            }

            let mut response: Option<String> = None;

            let executed = {
                let mut locked_app_state = app_state.lock().unwrap();
                if let Some(list_items)  = locked_app_state.cache.get_mut(&object_id) {
                    response = Some(get_browse_response(list_items));
                    true
                } else {
                    false
                }
            };
            if !executed {
                let list_item = {
                    let locked_app_state = app_state.lock().unwrap();
                    locked_app_state.item_map.get(&object_id).unwrap().clone()
                };
                let id_counter = {
                    let locked_app_state = app_state.lock().unwrap();
                    locked_app_state.id_counter
                };
                let x = read_directory(list_item.dir.clone().unwrap(), object_id, id_counter).await;
                let list_items= x.list_items;
                {
                    let mut locked_app_state = app_state.lock().unwrap();
                    locked_app_state.id_counter = x.id_counter;
                }
                for item in &list_items {
                    let mut locked_app_state = app_state.lock().unwrap();
                    locked_app_state.item_map.insert(item.id, item.clone());
                }
                let mut locked_app_state = app_state.lock().unwrap();
                locked_app_state.cache.insert(object_id, list_items.clone());
                response = Some(get_browse_response(&list_items));
            }

            let response = response.unwrap();

            log::info!("-----The Request Body-----\n{}\n", body_string);
            log::info!("Action: {}", action);
            log::info!("ObjectID: {}", object_id);
            log::info!("-----The Response Body-----\n{}\n", response);
            
            Ok(response)
        })
        .with(warp::reply::with::header("Content-type", "text/xml"))
        .with(warp::log::log("@agni/content-directory"))
        .boxed()
}

pub fn serve_directories() -> BoxedFilter<(impl Reply,)> {
    warp::any()
        .and(warp::fs::dir("/"))
        .boxed()
}