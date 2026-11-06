// Drawing over the title's Multiplayer screens: ViaFabricPlus's button (or Set
// version), the ping tooltip's version lines, and ViaFabricPlus's screens
// while one is open. Another screen closes them.
fn older_servers_screen(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use crate::game::native_hooks::older_servers_vfp_draw as draw;
    use horizon_client::net::{older_servers_mod as m, older_servers_vfp as v};
    if !m::on() || !draw::ours(screen.id) {
        if v::page().is_some() {
            v::close();
        }
        return;
    }
    draw::draw(game, batch, screen.id, [screen.rect[2], screen.rect[3]], screen.mouse);
}
