use std::net::UdpSocket;
use std::sync::Arc;
use crate::util::get_local_ip;
use uuid::Uuid;
use std::net::IpAddr;

pub fn get_broadcast_presence_func(uuid: Uuid) -> impl Fn() {
    let socket = UdpSocket::bind("[::]:0").unwrap();
    socket.connect("239.255.255.250:1900").unwrap();
    // socket.connect("[FF02::C]:1900").unwrap();
    // socket.connect("[FF05::C]:1900").unwrap();
    // socket.connect("[FF08::C]:1900").unwrap();
    // socket.connect("[FF0E::C]:1900").unwrap();
    let socket = Arc::new(socket);
    
    
    
    let make_msg = |ip:IpAddr,nt: String, usn: String| format!("\
        NOTIFY * HTTP/1.1\r\n\
        HOST: 239.255.255.250:1900\r\n\
        NT: {}\r\n\
        NTS: ssdp:alive\r\n\
        LOCATION: http://{}:3030/root.xml\r\n\
        USN: {}\r\n\
        CACHE-CONTROL: max-age=1800\r\n\
        SERVER: somesystem, UPnP/1.0, agni/1.0\r\n\
        \r\n",
        &nt,
        ip,
        &usn).into_bytes();
    
    let make_dup = move |ip,uuid_urn: String, nt: String| make_msg(ip, nt.clone(), format!("{}::{}", &uuid_urn, nt));
    
    let msg_root = move |ip, uuid_urn | make_dup(ip,uuid_urn,"upnp:rootdevice".to_owned());
    let msg_mediaserver = move |ip, uuid_urn |make_dup(ip,uuid_urn,"urn:schemas-upnp-org:device:MediaServer:1".to_owned());
    let msg_contentdir = move |ip, uuid_urn |make_dup(ip,uuid_urn,"urn:schemas-upnp-org:service:ContentDirectory:1".to_owned());
    let msg_connectionmanager = move |ip, uuid_urn | make_dup(ip,uuid_urn,"urn:schemas-upnp-org:service:ConnectionManager:1".to_owned());
    let msg_uuid = move |ip,uuid_urn: String | make_msg(ip, uuid_urn.clone(), uuid_urn.clone());
    
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
        let ips = get_local_ip();
        let uuid_urn = format!("uuid:{}", uuid);

        // log::info!("PUBLIC IP : {:#?}", ips);

        for ip in ips {
            for _ in 0..3 {
                broadcast_message("uuid", &msg_uuid(ip, uuid_urn.clone())).unwrap();
                broadcast_message("root", &msg_root(ip,uuid_urn.clone())).unwrap();
                broadcast_message("mediaserver", &msg_mediaserver(ip,uuid_urn.clone())).unwrap();
                broadcast_message("connectionmanager", &msg_connectionmanager(ip,uuid_urn.clone())).unwrap();
                broadcast_message("contentdir", &msg_contentdir(ip,uuid_urn.clone())).unwrap();
            };
        }
    };

    broadcast_presence
}