use warp::{Filter, filters::BoxedFilter, Reply};
use bytes::Bytes;
use quick_xml::Reader;
use crate::types;
    
const ROOT_XML: &str = include_str!("root.xml");
const CONTENT_DESC_XML: &str = include_str!("content_desc.xml");
const SOAP_ACTION :&str = "Soapaction";


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
            let xml = Reader::from_str(&body_string);

            log::error!("{}", action);
            log::error!("\n{}", body_string);

            warp::reply()
        })
        .boxed()
}