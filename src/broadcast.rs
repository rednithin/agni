use crate::util::get_local_ip;
use std::net::{IpAddr, Ipv4Addr};
use tokio::net::UdpSocket;
use uuid::Uuid;

pub async fn broadcast_message<'a>(desc: &'a str, data: &'a [u8], unicast_ip: &Option<String>) {
    let mut socket = UdpSocket::bind("[::]:0").await.unwrap();
    let addr = if let Some(x) = unicast_ip {
        x.to_owned()
    } else {
        "239.255.255.250:1900".to_string()
    };
    if !addr.contains("239.255.255.250:1900") {
        // println!("To Address: {:?}\nMessage:\n {:?}\n", addr, String::from_utf8_lossy(data));
    }
    socket
        .send_to(data, addr)
        .await
        .map(|bytes_written| {
            if bytes_written != data.len() {
                eprintln!("W: sending of {} truncated.", desc);
            }
        })
        .unwrap();
}

pub async fn broadcast_presence<'a>(uuid: Uuid, unicast_ip: Option<String>) {
    let make_msg = |ip: IpAddr, nt: &str, usn: &str| {
        format!(
            "\
NOTIFY * HTTP/1.1\r\n\
HOST: 239.255.255.250:1900\r\n\
NT: {}\r\n\
NTS: ssdp:alive\r\n\
LOCATION: http://{}:3030/root.xml\r\n\
USN: {}\r\n\
CACHE-CONTROL: max-age=1800\r\n\
SERVER: Linux/5.8, UPnP/1.0, agni/1.0\r\n\
\r\n",
            nt, ip, usn
        )
        .into_bytes()
    };

    let make_dup = move |ip, uuid_urn: &str, nt| make_msg(ip, nt, &format!("{}::{}", uuid_urn, nt));

    let msg_root = move |ip, uuid_urn: &str| make_dup(ip, uuid_urn, "upnp:rootdevice");
    let msg_mediaserver = move |ip, uuid_urn: &str| {
        make_dup(ip, uuid_urn, "urn:schemas-upnp-org:device:MediaServer:1")
    };
    let msg_contentdir = move |ip, uuid_urn: &str| {
        make_dup(
            ip,
            uuid_urn,
            "urn:schemas-upnp-org:service:ContentDirectory:1",
        )
    };
    let msg_connectionmanager = move |ip, uuid_urn: &str| {
        make_dup(
            ip,
            uuid_urn,
            "urn:schemas-upnp-org:service:ConnectionManager:1",
        )
    };
    let msg_uuid = |ip, uuid_urn: &str| make_msg(ip, uuid_urn, uuid_urn);

    let broadcast = || async move {
        // println!("Broadcasted");
        let ips = get_local_ip();
        let uuid_urn = format!("uuid:{}", uuid);

        // log::info!("PUBLIC IP : {:#?}", ips);

        for ip in ips {
            for _ in 0..3i32 {
                broadcast_message("uuid", &msg_uuid(ip, &uuid_urn), &unicast_ip).await;
                broadcast_message("root", &msg_root(ip, &uuid_urn), &unicast_ip).await;
                broadcast_message("mediaserver", &msg_mediaserver(ip, &uuid_urn), &unicast_ip)
                    .await;
                broadcast_message(
                    "connectionmanager",
                    &msg_connectionmanager(ip, &uuid_urn),
                    &unicast_ip,
                )
                .await;
                broadcast_message("contentdir", &msg_contentdir(ip, &uuid_urn), &unicast_ip).await;
            }
        }
    };

    broadcast().await;
}

pub async fn reply_presence<'a>(uuid: Uuid, unicast_ip: Option<String>) {
    let make_msg = |ip: IpAddr, nt: &str, usn: &str| {
        format!(
            "\
HTTP/1.1 200 OK\r\n\
CACHE-CONTROL: max-age=1800\r\n\
DATE: {}\r\n\
EXT:\r\n\
LOCATION: http://{}:3030/root.xml\r\n\
SERVER: Linux/5.8, UPnP/1.0, agni/1.0\r\n\
ST: {}\r\n\
USN: {}\r\n\
\r\n",
            chrono::offset::Utc::now().format("%a, %m %b %Y %H:%M:%S GMT"),
            ip,
            nt,
            usn
        )
        .into_bytes()
    };

    let make_dup = move |ip, uuid_urn: &str, nt| make_msg(ip, nt, &format!("{}::{}", uuid_urn, nt));

    let msg_root = move |ip, uuid_urn: &str| make_dup(ip, uuid_urn, "upnp:rootdevice");
    let msg_mediaserver = move |ip, uuid_urn: &str| {
        make_dup(ip, uuid_urn, "urn:schemas-upnp-org:device:MediaServer:1")
    };
    let msg_contentdir = move |ip, uuid_urn: &str| {
        make_dup(
            ip,
            uuid_urn,
            "urn:schemas-upnp-org:service:ContentDirectory:1",
        )
    };
    let msg_connectionmanager = move |ip, uuid_urn: &str| {
        make_dup(
            ip,
            uuid_urn,
            "urn:schemas-upnp-org:service:ConnectionManager:1",
        )
    };
    let msg_uuid = |ip, uuid_urn: &str| make_msg(ip, uuid_urn, uuid_urn);

    let broadcast = || async move {
        // println!("Broadcasted");
        let ips = get_local_ip();
        let uuid_urn = format!("uuid:{}", uuid);

        // log::info!("PUBLIC IP : {:#?}", ips);

        for ip in ips {
            for _ in 0..3i32 {
                broadcast_message("uuid", &msg_uuid(ip, &uuid_urn), &unicast_ip).await;
                broadcast_message("root", &msg_root(ip, &uuid_urn), &unicast_ip).await;
                broadcast_message("mediaserver", &msg_mediaserver(ip, &uuid_urn), &unicast_ip)
                    .await;
                broadcast_message(
                    "connectionmanager",
                    &msg_connectionmanager(ip, &uuid_urn),
                    &unicast_ip,
                )
                .await;
                broadcast_message("contentdir", &msg_contentdir(ip, &uuid_urn), &unicast_ip).await;
            }
        }
    };

    broadcast().await;
}

pub async fn listen_to_discover_messages(uuid: Uuid) {
    let mut socket = UdpSocket::bind("0.0.0.0:1900").await.unwrap();
    socket
        .join_multicast_v4(Ipv4Addr::new(239, 255, 255, 250), Ipv4Addr::new(0, 0, 0, 0))
        .unwrap();

    loop {
        let mut buf = [0; 2048];

        match socket.recv_from(&mut buf).await {
            Ok((received, addr)) => {
                let s = String::from_utf8_lossy(&buf[..received]);
                if s.contains("M-SEARCH")
                    && s.contains("ssdp:discover")
                    && s.contains("ContentDirectory")
                {
                    // println!("From Address: {:?}\nMessage:\n {:?}\n", addr.to_string(), s);
                    reply_presence(uuid.clone(), Some(addr.to_string())).await;
                }
            }
            Err(e) => eprintln!("recv function failed: {:?}", e),
        };
    }
}
