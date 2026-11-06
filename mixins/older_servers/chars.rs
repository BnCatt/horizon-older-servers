// Typing goes into ViaFabricPlus's search box while its screen is open.
fn older_servers_chars(c: char, canceled: &mut bool) {
    if horizon_client::net::older_servers_vfp::typed(c) {
        *canceled = true;
    }
}
