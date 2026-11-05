// A click on the version button: left steps to the next version, right back.
fn older_servers_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use horizon_client::net::older_servers_mod as m;
    if screen.id != "multiplayer" || !m::on() || !(button == 0 || button == 1) {
        return;
    }
    let rect = m::button(game.native_text_width(&m::label()));
    if !m::inside(rect, screen.mouse) {
        return;
    }
    *canceled = true;
    let stored = m::step(button == 0);
    game.native_set_setting(m::ID, "version", &stored);
}
