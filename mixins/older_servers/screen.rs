// Çizim: Multiplayer ekranında sürüm butonu, sunucu satırlarında sürüm etiketi
// ve açıksa sürüm paneli; Edit/Add Server ekranında sunucu başına sürüm butonu.
fn older_servers_screen(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use horizon_client::net::{
        older_servers_mod as m, older_servers_panel as p, older_servers_skin as skin,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    static WARNED: AtomicBool = AtomicBool::new(false);
    if !m::on() {
        return;
    }
    let size = [screen.rect[2], screen.rect[3]];
    let tw = |s: &str| game.native_text_width(s);
    let draw = |batch: &mut horizon_client::hud::HudBatch, parts: &[skin::Draw]| {
        for part in parts {
            match part {
                skin::Draw::Fill(rect, argb) => game.native_fill(batch, *rect, *argb),
                skin::Draw::Text(text, at, rgb, shadow) => {
                    game.native_text(batch, text, *at, *rgb, *shadow)
                }
                skin::Draw::Sprite { name, src, dst } => {
                    if !game.native_sprite(batch, name, *src, *dst) {
                        // Atlas'ta yoksa (kaynak paketi?) düz renkli yedek.
                        if !WARNED.swap(true, Ordering::Relaxed) {
                            log::warn!(target: "horizon_older_servers", "no sprite {name}: drawing plain buttons");
                        }
                        game.native_fill(batch, *dst, 0xFF6E_6E6E);
                    }
                }
            }
        }
    };
    let multiplayer = || match game.title.as_ref().map(|t| &t.mode) {
        Some(horizon_client::title::Mode::Multiplayer(s)) => Some(s),
        _ => None,
    };
    if screen.id != "multiplayer" {
        // Başka ekrana geçince panel kapanır.
        if let Ok(mut panel) = p::PANEL.lock() {
            panel.open = false;
        }
    }
    match screen.id {
        "multiplayer" => {
            let settings = m::settings();
            let panel = *p::PANEL.lock().unwrap();
            if settings.list_tags
                && let Some(s) = multiplayer()
            {
                for row in horizon_client::multiplayer::older_servers_rows::visible(s, size) {
                    let Some((text, rgb)) = m::tag_for(&row.address, m::pinged(&row.address)) else {
                        continue;
                    };
                    let x = row.at[0] + tw(&row.name) + 6.0;
                    if x + tw(&text) <= row.limit {
                        game.native_text(batch, &text, [x, row.at[1]], rgb, true);
                    }
                }
            }
            if !settings.hide_button {
                let label = m::label();
                let width = tw(&label);
                let rect = m::button(size, width);
                let state = if panel.open {
                    skin::ButtonState::Selected
                } else if m::inside(rect, screen.mouse) {
                    skin::ButtonState::Hover
                } else {
                    skin::ButtonState::Normal
                };
                draw(batch, &skin::button(rect, state, &label, width));
            }
            if panel.open {
                let (parts, _) = p::layout(
                    size,
                    &m::choices(),
                    m::choice_index(),
                    panel.scroll,
                    screen.mouse,
                    settings.auto_detect,
                    &tw,
                );
                draw(batch, &parts);
            }
        }
        "edit_server" | "add_server" => {
            let Some(address) = multiplayer()
                .and_then(horizon_client::multiplayer::older_servers_rows::edit_address)
            else {
                return;
            };
            let label = m::server_label(address);
            let width = tw(&label);
            let rect = m::edit_button(size, width);
            let state = if m::inside(rect, screen.mouse) {
                skin::ButtonState::Hover
            } else {
                skin::ButtonState::Normal
            };
            draw(batch, &skin::button(rect, state, &label, width));
        }
        _ => {}
    }
}
