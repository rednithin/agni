use std::net::UdpSocket;
use std::sync::Arc;
use crate::util::get_local_ip;
use uuid::Uuid;

pub fn get_broadcast_presence_func(uuid: Uuid) -> impl Fn() {
    let socket = UdpSocket::bind("[::]:0").unwrap();
    socket.connect("239.255.255.250:1900").unwrap();
    let socket = Arc::new(socket);
    
    let ip = get_local_ip();
    let uuid_urn = format!("uuid:{}", uuid);

    log::info!("PUBLIC IP : {}", ip);
    
    let make_msg = |nt, usn: &str| format!("\
        NOTIFY * HTTP/1.1\r\n\
        HOST: 239.255.255.250:1900\r\n\
        NT: {}\r\n\
        NTS: ssdp:alive\r\n\
        LOCATION: http://{}:3030/root.xml\r\n\
        USN: {}\r\n\
        CACHE-CONTROL: max-age=1800\r\n\
        SERVER: somesystem, UPnP/1.0, rustyupnp/1.0\r\n\
        \r\n",
        nt,
        ip,
        usn).into_bytes();
    
    let make_dup = |nt| make_msg(nt, format!("{}::{}", uuid_urn, nt).as_str());
    
    let msg_root = make_dup("upnp:rootdevice");
    let msg_mediaserver = make_dup("urn:schemas-upnp-org:device:MediaServer:1");
    let msg_contentdir = make_dup("urn:schemas-upnp-org:service:ContentDirectory:1");
    let msg_connectionmanager = make_dup("urn:schemas-upnp-org:service:ConnectionManager:1");
    let msg_uuid = make_msg(&uuid_urn, &uuid_urn);
    
    let broadcast_message = move |desc, data: &[u8]| {
        socket
            .send(data)
            .map(|bytes_written| 
                if bytes_written != data.len() {
                    eprintln!("W: sending of {} truncated.", desc); 
                }
            )
    };
    
    let broadcast_presence = move || {
        // println!("Broadcasted");
        for _ in 0..3 {
            broadcast_message("uuid", &msg_uuid).unwrap();
            broadcast_message("root", &msg_root).unwrap();
            broadcast_message("mediaserver", &msg_mediaserver).unwrap();
            broadcast_message("connectionmanager", &msg_connectionmanager).unwrap();
            broadcast_message("contentdir", &msg_contentdir).unwrap();
        };
    };

    broadcast_presence
}