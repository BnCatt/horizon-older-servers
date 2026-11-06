// The wheel scrolls ViaFabricPlus's list while its screen is open.
fn older_servers_scroll(
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    scroll: [f64; 2],
    canceled: &mut bool,
) {
    use horizon_client::net::older_servers_vfp as v;
    if v::page().is_none() {
        return;
    }
    *canceled = true;
    v::wheel([screen.rect[2], screen.rect[3]], scroll[1] as f32);
}
