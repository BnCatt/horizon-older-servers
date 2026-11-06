// Sunucu listesi ping'lenirken her sunucunun protokol numarası da okunur (sürüm
// etiketi için). Horizon'un ping'i protokolü saklamadığından ayrı, hafif bir
// durum isteği atılır; listede etiket kapalıysa hiç atılmaz.
fn older_servers_ping_all(servers: &[ServerEntry]) {
    use crate::net::older_servers_mod as m;
    if !m::on() || !m::settings().show_advertised {
        return;
    }
    for server in servers {
        let address = server.ip.clone();
        let _ = std::thread::Builder::new()
            .name("horizon-older-servers-ping".into())
            .spawn(move || {
                if let Some(p) = crate::net::ping(&address, std::time::Duration::from_secs(5))
                    .ok()
                    .and_then(|(json, _)| json["version"]["protocol"].as_i64())
                    .and_then(|p| i32::try_from(p).ok())
                {
                    m::record_ping(&address, p);
                }
            });
    }
}
