// Clicks: ViaFabricPlus's button opens its screen (Force Version on Add/Edit
// Server); while a screen of it is open every click is its own.
fn older_servers_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use crate::game::native_hooks::older_servers_vfp_draw as draw;
    use horizon_client::net::{older_servers_mod as m, older_servers_vfp as v};
    if !m::on() || !draw::ours(screen.id) {
        return;
    }
    let size = [screen.rect[2], screen.rect[3]];
    if v::page().is_some() {
        *canceled = true;
        if button != 0 {
            return;
        }
        let out = v::click(size, screen.mouse, &game.gui.atlas);
        for (key, value) in out.store {
            game.native_set_setting(m::ID, key, &value);
        }
        if let Some(url) = out.open {
            crate::game::packs::open_uri(&url);
        }
        if out.logs {
            crate::game::packs::open_folder(&game.native_instance_dir().join("logs"));
        }
        return;
    }
    if button != 0 {
        return;
    }
    let Some((rect, _)) = draw::screen_button(game, screen.id, size) else {
        return;
    };
    if v::inside(rect, screen.mouse) {
        *canceled = true;
        match screen.id {
            "edit_server" | "add_server" => v::open(v::Page::Force, draw::edit_address(game)),
            _ => v::open(v::Page::Protocols, None),
        }
    }
}
