// Bir sunucuya hangi sürümle bağlanılacağı: sunucuya sabitlenen sürüm, yoksa
// listeden seçilen, o da yoksa (otomatik algılama açıkken) sunucunun ping'i;
// kapalıyken 26.3. Seçilen sürüm F3 satırı için kaydedilir. 26.3'ün kendisi
// çeviri değildir (None).
fn older_servers_pick(
    address: &str,
) -> std::ops::ControlFlow<Option<horizon_protocol::adapter::ServerVersion>> {
    use crate::net::older_servers_mod as m;
    use horizon_protocol::adapter::ServerVersion;
    if !m::on() {
        return std::ops::ControlFlow::Continue(());
    }
    let (version, source) = match m::decide(m::forced(address), m::choice(), m::settings().auto_detect) {
        m::Decision::Use(v, source) => (v, source),
        m::Decision::Detect => {
            // Horizon'un kendi `older_server`'ı gibi: durum ping'i ve protokol numarası.
            let protocol = ping(address, std::time::Duration::from_secs(5))
                .ok()
                .and_then(|(json, _)| json["version"]["protocol"].as_i64())
                .and_then(|p| i32::try_from(p).ok());
            if let Some(p) = protocol {
                m::record_ping(address, p);
            }
            (m::detected(protocol), m::Source::Detected)
        }
    };
    m::set_connected(Some((version, source)));
    log::info!(
        target: "horizon_older_servers",
        "{address}: {} ({}, {source:?})",
        version.name(),
        version.protocol()
    );
    std::ops::ControlFlow::Break(Some(version).filter(|v| *v != ServerVersion::Current))
}
