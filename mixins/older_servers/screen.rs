// The version button, top left of the Multiplayer screen.
fn older_servers_screen(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use horizon_client::net::older_servers_mod as m;
    if screen.id != "multiplayer" || !m::on() {
        return;
    }
    let label = m::label();
    let width = game.native_text_width(&label);
    let rect = m::button(width);
    let hover = m::inside(rect, screen.mouse);
    // Vanilla's button look: a dark face, a lighter edge when hovered.
    let edge = if hover { 0xFFFF_FFFF } else { 0xFF00_0000 };
    game.native_fill(batch, rect, edge);
    game.native_fill(
        batch,
        [rect[0] + 1.0, rect[1] + 1.0, rect[2] - 2.0, rect[3] - 2.0],
        if hover { 0xFF6E_6E6E } else { 0xFF4A_4A4A },
    );
    let x = rect[0] + (rect[2] - width) / 2.0;
    game.native_text(batch, &label, [x, rect[1] + 6.0], 0xFF_FFFF, true);
    if hover {
        let tip = "Sol tık: sonraki · Sağ tık: önceki";
        let w = game.native_text_width(tip) + 6.0;
        let at = [rect[0], rect[1] + rect[3] + 3.0];
        game.native_fill(batch, [at[0], at[1], w, 13.0], 0xE010_0010);
        game.native_text(batch, tip, [at[0] + 3.0, at[1] + 3.0], 0xFF_FFFF, true);
    }
}
