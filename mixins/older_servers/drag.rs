// A held scrollbar follows the pointer.
fn older_servers_drag(screen: &crate::game::native_hooks::NativeScreenView<'_>, canceled: &mut bool) {
    use horizon_client::net::older_servers_vfp as v;
    if v::page().is_none() {
        return;
    }
    *canceled = true;
    v::drag([screen.rect[2], screen.rect[3]], screen.mouse[1]);
}
