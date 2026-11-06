// Letting go of the mouse lets go of the scrollbar.
fn older_servers_released(canceled: &mut bool) {
    use horizon_client::net::older_servers_vfp as v;
    if v::page().is_none() {
        return;
    }
    *canceled = true;
    v::release();
}
