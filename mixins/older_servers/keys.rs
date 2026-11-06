// Keys while ViaFabricPlus's screen is open: Escape goes back a screen (the
// Multiplayer screen stays), the search box takes the rest.
fn older_servers_keys(
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    key: &str,
    modifiers: u32,
    canceled: &mut bool,
) {
    use horizon_client::net::older_servers_vfp as v;
    let _ = screen;
    // GLFW modifier bits: 1 shift, 2 control.
    if v::key(key, modifiers & 1 != 0, modifiers & 2 != 0) {
        *canceled = true;
    }
}
