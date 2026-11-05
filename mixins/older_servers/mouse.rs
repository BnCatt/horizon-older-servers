// Tıklamalar: Multiplayer'da sürüm butonu paneli açar; panel açıkken her tık
// panelindir (alttaki butonlara sızmaz). Edit/Add Server'da sürüm butonu
// sunucunun sürümünü döndürür (sol: ileri, sağ: geri).
fn older_servers_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use horizon_client::net::{older_servers_mod as m, older_servers_panel as p};
    if !m::on() {
        return;
    }
    let size = [screen.rect[2], screen.rect[3]];
    match screen.id {
        "multiplayer" => {
            let mut panel = p::PANEL.lock().unwrap();
            let choices = m::choices();
            if panel.open {
                *canceled = true;
                if button != 0 {
                    return;
                }
                let settings = m::settings();
                let (_, hits) = p::layout(
                    size,
                    &choices,
                    m::choice_index(),
                    panel.scroll,
                    screen.mouse,
                    settings.auto_detect,
                    &|s: &str| game.native_text_width(s),
                );
                match p::click(&hits, screen.mouse) {
                    p::PanelEvent::Select(i) => {
                        let stored = m::set_choice(i);
                        game.native_set_setting(m::ID, m::VERSION_KEY, &stored);
                        panel.open = false;
                    }
                    p::PanelEvent::ToggleAuto => {
                        for (key, value) in m::toggle_auto() {
                            game.native_set_setting(m::ID, key, &value);
                        }
                    }
                    p::PanelEvent::Close | p::PanelEvent::Outside => panel.open = false,
                    p::PanelEvent::None => {}
                }
                return;
            }
            if m::settings().hide_button || !(button == 0 || button == 1) {
                return;
            }
            let rect = m::button(size, game.native_text_width(&m::label()));
            if m::inside(rect, screen.mouse) {
                *canceled = true;
                let visible = p::geometry(size, choices.len()).visible;
                panel.open = true;
                panel.scroll = p::scroll_to_show(m::choice_index(), 0, choices.len(), visible);
            }
        }
        "edit_server" | "add_server" => {
            if !(button == 0 || button == 1) {
                return;
            }
            let Some(horizon_client::title::Mode::Multiplayer(s)) =
                game.title.as_ref().map(|t| &t.mode)
            else {
                return;
            };
            let Some(address) = horizon_client::multiplayer::older_servers_rows::edit_address(s)
            else {
                return;
            };
            let rect = m::edit_button(size, game.native_text_width(&m::server_label(address)));
            if m::inside(rect, screen.mouse) {
                *canceled = true;
                let stored = m::cycle_forced(address, button == 0);
                game.native_set_setting(m::ID, m::SERVERS_KEY, &stored);
            }
        }
        _ => {}
    }
}
