// The version to speak to a server: the one picked on the Multiplayer screen,
// or Horizon's own status ping on Auto. The pinned version itself is no
// translation (None).
fn older_servers_pick(
    address: &str,
) -> std::ops::ControlFlow<Option<horizon_protocol::adapter::ServerVersion>> {
    use crate::net::older_servers_mod as m;
    use horizon_protocol::adapter::ServerVersion;
    let _ = address;
    if !m::on() {
        return std::ops::ControlFlow::Continue(());
    }
    match m::choice() {
        None => std::ops::ControlFlow::Continue(()),
        Some(v) => std::ops::ControlFlow::Break(Some(v).filter(|v| *v != ServerVersion::Current)),
    }
}
