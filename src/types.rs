use strong_xml::{XmlRead, XmlWrite};
use lru_cache::LruCache;
use std::collections::HashMap;

pub const XMLNS_ENVELOPE: &str = "http://schemas.xmlsoap.org/soap/envelope/";
pub const ENVELOPE_ENCODING_STYLE: &str = "http://schemas.xmlsoap.org/soap/encoding/";
pub const XMLNS_CONTENT_DIRECTORY: &str = "urn:schemas-upnp-org:service:ContentDirectory:1";

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "s:Envelope")]
pub struct Envelope {
    #[xml(attr = "xmlns:s")]
    pub xmlns: String,
    #[xml(attr = "s:encodingStyle")]
    pub encoding_style: String,
    #[xml(child = "s:Body")]
    pub body: Body,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "s:Body")]
pub struct Body {
    #[xml(child = "u:BrowseResponse")]
    pub browse_response: BrowseResponse,
    #[xml(attr = "xmlns:s")]
    pub xmlns: String,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "u:BrowseResponse")]
pub struct BrowseResponse {
    #[xml(flatten_text = "Result")]
    pub result: String,
    #[xml(flatten_text = "NumberReturned")]
    pub number_returned: u64,
    #[xml(flatten_text = "TotalMatches")]
    pub total_matches: u64,
    #[xml(flatten_text = "UpdateID")]
    pub update_id: u64,
}

pub const XMLNS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const XMLNS_UPNP: &str = "urn:schemas-upnp-org:metadata-1-0/upnp/";
pub const XMLNS_DIDL: &str = "urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/";

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "DIDL-Lite")]
pub struct DidlLite {
    #[xml(attr = "xmlns:dc")]
    pub xmlns_dc: String,
    #[xml(attr = "xmlns:upnp")]
    pub xmlns_upnp: String,
    #[xml(attr = "xmlns")]
    pub xmlns: String,
    #[xml(child = "container", child="item")]
    pub list_items: Vec<ListItem>
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
pub enum ListItem {
    #[xml(tag = "container")]
    Container(Container),
    #[xml(tag = "item")]
    Item(Item),
}

#[derive(PartialEq, Debug, Clone)]
pub struct ListItemWrapper {
    pub list_item: ListItem,
    pub id: u64,
    pub dir: Option<String>
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "container")]
pub struct Container {
    #[xml(attr = "id")]
    pub id: u64,
    #[xml(attr = "parentId")]
    pub parent_id: u64,
    #[xml(flatten_text = "dc:title")]
    pub title: String,
    #[xml(flatten_text = "upnp:class")]
    pub class: String,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "item")]
pub struct Item {
    #[xml(attr = "id")]
    pub id: u64,
    #[xml(attr = "parentId")]
    pub parent_id: u64,
    #[xml(flatten_text = "dc:title")]
    pub title: String,
    #[xml(flatten_text = "upnp:class")]
    pub class: String,
    #[xml(child = "res")]
    pub res: Res,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "res")]
pub struct Res {
    #[xml(attr = "protocolInfo")]
    pub protocol_info: String,
    #[xml(text)]
    pub content: String,
}

pub struct AppState {
    pub cache: LruCache<u64,Vec<ListItemWrapper>>,
    pub item_map: HashMap<u64,ListItemWrapper>,
    pub id_counter: u64,
}