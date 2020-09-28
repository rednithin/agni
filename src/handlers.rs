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

            let response = format!(r#"<?xml version="1.0" encoding="utf-8"?>
            <s:Envelope
                xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                <s:Body>
                    <u:BrowseResponse
                        xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
                        <Result>
                            &lt;DIDL-Lite
                                xmlns:dc=&quot;http://purl.org/dc/elements/1.1/&quot;
                                xmlns:upnp=&quot;urn:schemas-upnp-org:metadata-1-0/upnp/&quot;
                                xmlns=&quot;urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/&quot;&gt;
                                &lt;container id=&quot;1&quot; parentID=&quot;0&quot; childCount=&quot;2&quot; restricted=&quot;false&quot;&gt;
                                    &lt;dc:title&gt;My Music&lt;/dc:title&gt;
                                    &lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;
                                    &lt;upnp:storageUsed&gt;730000&lt;/upnp:storageUsed&gt;
                                    &lt;upnp:writeStatus&gt;WRITABLE&lt;/upnp:writeStatus&gt;
                                    &lt;upnp:searchClass includeDerived=&quot;false&quot;&gt;object.container.album.musicAlbum&lt;/upnp:searchClass&gt;
                                    &lt;upnp:searchClass includeDerived=&quot;false&quot;&gt;object.item.audioItem.musicTrack&lt;/upnp:searchClass&gt;
                                    &lt;upnp:createClass includeDerived=&quot;false&quot;&gt;object.container.album.musicAlbum&lt;/upnp:createClass&gt;
                                &lt;/container&gt;
                                &lt;container id=&quot;2&quot; parentID=&quot;0&quot; childCount=&quot;2&quot; restricted=&quot;false&quot;&gt;
                                    &lt;dc:title&gt;My Photos&lt;/dc:title&gt;
                                    &lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;
                                    &lt;upnp:storageUsed&gt;177000&lt;/upnp:storageUsed&gt;
                                    &lt;upnp:writeStatus&gt;WRITABLE&lt;/upnp:writeStatus&gt;
                                    &lt;upnp:searchClass includeDerived=&quot;false&quot;&gt;object.container.album.photoAlbum&lt;/upnp:searchClass&gt;
                                    &lt;upnp:searchClass includeDerived=&quot;false&quot;&gt;object.item.imageItem.photo&lt;/upnp:searchClass&gt;
                                    &lt;upnp:createClass includeDerived=&quot;false&quot;&gt;object.container.album.photoAlbum&lt;/upnp:createClass&gt;
                                &lt;/container&gt;
                                &lt;container id=&quot;30&quot; parentID=&quot;0&quot; childCount=&quot;2&quot; restricted=&quot;false&quot;&gt;
                                    &lt;dc:title&gt;Album Art&lt;/dc:title&gt;
                                    &lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;
                                    &lt;upnp:storageUsed&gt;40000&lt;/upnp:storageUsed&gt;
                                    &lt;upnp:writeStatus&gt;WRITABLE&lt;/upnp:writeStatus&gt;
                                    &lt;upnp:searchClass name=&quot;Vendor Album Art&quot; includeDerived=&quot;true&quot;&gt;object.item.imageItem.photo.vendorAlbumArt&lt;/upnp:searchClass&gt;
                                    &lt;upnp:createClass includeDerived=&quot;true&quot;&gt;object.item.imageItem.photo.vendorAlbumArt&lt;/upnp:createClass&gt;
                                &lt;/container&gt;
                            &lt;/DIDL-Lite&gt;
                        </Result>
                        <NumberReturned>2</NumberReturned>
                        <TotalMatches>2</TotalMatches>
                        <UpdateId>10</UpdateId>
                    </u:BrowseResponse>
                </s:Body>
            </s:Envelope>"#);

            response
        })
        .with(warp::reply::with::header("Content-type", "text/xml"))
        .boxed()
}