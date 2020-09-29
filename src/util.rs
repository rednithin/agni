use pnet::datalink;

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