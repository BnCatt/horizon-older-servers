// Joining older servers is on while the mod is: no HORIZON_OLDER_SERVERS needed.
fn older_servers_enabled_head() -> std::ops::ControlFlow<bool> {
    if crate::net::older_servers_mod::on() {
        std::ops::ControlFlow::Break(true)
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
