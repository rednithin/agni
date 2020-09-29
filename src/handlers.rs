use warp::{Filter, filters::BoxedFilter, Reply};
use bytes::Bytes;
use crate::types;
    
const ROOT_XML: &str = include_str!("root.xml");
const CONTENT_DESC_XML: &str = include_str!("content_desc.xml");
const SOAP_ACTION :&str = "Soapaction";

use types::{
    Envelope,
    Body,
    BrowseResponse,
    DidlLite,
    Container,
    XMLNS_DC,
    XMLNS_DIDL,
    XMLNS_UPNP,
    ENVELOPE_ENCODING_STYLE,
    XMLNS_ENVELOPE,
    XMLNS_CONTENT_DIRECTORY,
};

fn get_browse_response() -> String {
    let didl_result = DidlLite {
        xmlns_dc: XMLNS_DC.to_string(),
        xmlns_upnp: XMLNS_UPNP.to_string(),
        xmlns: XMLNS_DIDL.to_string(),
        containers: vec![
            Container {
                id: 1,
                parent_id: 0,
                title: "My Music".to_string(),
                class: "object.container.storageFolder".to_string(),
            }
        ]
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

pub fn root_handler() -> BoxedFilter<(impl Reply,)> {
    warp::any()
        .and(warp::get())
        .and(warp::path!("root.xml"))
        .map(|| {
            ROOT_XML
                .replace("{name}", "Rednithin")
                .replace("{uuid}", "06289e13-a832-4d76-be0b-00151d439863")
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

pub fn content_handler() -> BoxedFilter<(impl Reply,)> {
    warp::any()
        .and(warp::post())
        .and(warp::path!("content" / "control"))
        .and(warp::header::<String>(SOAP_ACTION))
        .and(warp::body::bytes())
        .map(|soap_action_header: String, body: Bytes| {
            let action = match soap_action_header.trim_matches('"').split("#").collect::<Vec<&str>>().get(1) {
                Some(&x) => x,
                _ => "Error"
            };

            let body_vec = body.to_vec();
            let body_string = String::from_utf8_lossy(&body_vec);

            log::info!("-----The Request Body-----\n{}\n", body_string);
            log::info!("Action: {}", action);

            let response = get_browse_response();
            log::info!("-----The Response Body-----\n{}\n", response);
            response
        })
        .with(warp::reply::with::header("Content-type", "text/xml"))
        .boxed()
}