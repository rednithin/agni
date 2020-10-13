use crate::types::{
    AppState, Body, BrowseResponse, DidlLite, Envelope, ListItemWrapper, ENVELOPE_ENCODING_STYLE,
    XMLNS_CONTENT_DIRECTORY, XMLNS_DC, XMLNS_DIDL, XMLNS_ENVELOPE, XMLNS_UPNP,
};
use crate::util::read_directory;
use actix_files::NamedFile;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result as ActixResult};
use bytes::Bytes;
use log;
use std::sync::{Arc, Mutex};

const ROOT_XML: &str = include_str!("root.xml");
const CONTENT_DESC_XML: &str = include_str!("content_desc.xml");
const CONNECTION_DESC_XML: &str = include_str!("connection_desc.xml");
const SOAP_ACTION: &str = "Soapaction";

fn get_browse_response(list_items: &Vec<ListItemWrapper>) -> String {
    let didl_result = DidlLite {
        xmlns_dc: XMLNS_DC.to_string(),
        xmlns_upnp: XMLNS_UPNP.to_string(),
        xmlns: XMLNS_DIDL.to_string(),
        list_items: list_items.iter().map(|x| x.list_item.clone()).collect(),
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
            },
        },
    };
    use strong_xml::XmlWrite;

    let didl_result = didl_result
        .to_string()
        .unwrap()
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    response
        .to_string()
        .unwrap()
        .replace("{didl-result}", &didl_result)
}

#[get("/root.xml")]
async fn root_handler(app_state: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let uuid_string = app_state.lock().unwrap().uuid.clone().to_string();
    let body = ROOT_XML
        .replace("{name}", "Actix-Rednithin-Dev")
        .replace("{uuid}", &uuid_string.clone());
    HttpResponse::Ok().content_type("text/xml").body(body)
}

#[get("/content/desc.xml")]
async fn content_desc_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/xml")
        .body(CONTENT_DESC_XML)
}

#[post("/content/control")]
async fn content_handler(
    app_state: web::Data<Arc<Mutex<AppState>>>,
    bytes: Bytes,
    req: HttpRequest,
) -> HttpResponse {
    let hostname = req.connection_info().host().to_owned();
    let soap_action_header = req
        .headers()
        .get(SOAP_ACTION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let action = match soap_action_header
        .trim_matches('"')
        .split("#")
        .collect::<Vec<&str>>()
        .get(1)
    {
        Some(&x) => x,
        _ => "",
    };

    let body_vec = bytes.to_vec();
    let body_string = String::from_utf8_lossy(&body_vec);

    let xml_doc = roxmltree::Document::parse(&body_string).unwrap();
    let elem = xml_doc
        .descendants()
        .find(|x| x.tag_name().name() == "ObjectID")
        .unwrap();
    let object_id = if let Ok(x) = elem.text().unwrap().to_string().parse::<u64>() {
        x
    } else {
        std::u64::MAX
    };

    if object_id == std::u64::MAX {
        return HttpResponse::NotFound().body("Lol");
    }

    let mut response: Option<String> = None;

    let executed = {
        let mut locked_app_state = app_state.lock().unwrap();
        if let Some(list_items) = locked_app_state.cache.get_mut(&object_id) {
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
        let x = read_directory(
            hostname,
            list_item.dir.clone().unwrap(),
            object_id,
            id_counter,
        )
        .await;
        let list_items = x.list_items;
        {
            let mut locked_app_state = app_state.lock().unwrap();
            locked_app_state.id_counter = x.id_counter;
        }
        for item in &list_items {
            let mut locked_app_state = app_state.lock().unwrap();
            locked_app_state.item_map.insert(item.id, item.clone());
        }
        // let mut locked_app_state = app_state.lock().unwrap();
        // locked_app_state.cache.insert(object_id, list_items.clone());
        response = Some(get_browse_response(&list_items));
    }

    let response = response.unwrap();

    log::info!("-----The Request Body-----\n{}\n", body_string);
    log::info!("Action: {}", action);
    log::info!("ObjectID: {}", object_id);
    log::info!("-----The Response Body-----\n{}\n", response);

    HttpResponse::Ok().content_type("text/xml").body(response)
}

#[get("/connection/desc.xml")]
async fn connection_desc_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/xml")
        .body(CONNECTION_DESC_XML)
}

#[get("/agni-files{filename:.*}")]
async fn serve_directories(req: HttpRequest) -> ActixResult<NamedFile> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root_handler);
    cfg.service(content_desc_handler);
    cfg.service(content_handler);
    cfg.service(connection_desc_handler);
    cfg.service(serve_directories);
}
