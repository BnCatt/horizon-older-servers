// Panel açıkken tuşlar: Esc kapatır (Multiplayer ekranı kapanmaz), yukarı/aşağı
// seçimi kaydırır, Enter seçer; diğer tuşlar yutulur.
fn older_servers_keys(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    key: &str,
    canceled: &mut bool,
) {
    use horizon_client::net::{older_servers_mod as m, older_servers_panel as p};
    if screen.id != "multiplayer" || !m::on() {
        return;
    }
    let mut panel = p::PANEL.lock().unwrap();
    if !panel.open {
        return;
    }
    *canceled = true;
    let n = m::choices().len();
    let visible = p::geometry([screen.rect[2], screen.rect[3]], n).visible;
    let at = m::choice_index();
    let moved = match p::key(key) {
        p::KeyAction::Close => {
            panel.open = false;
            return;
        }
        p::KeyAction::Select => {
            panel.open = false;
            return;
        }
        p::KeyAction::Up => at.saturating_sub(1),
        p::KeyAction::Down => (at + 1).min(n - 1),
        p::KeyAction::Ignore => return,
    };
    let stored = m::set_choice(moved);
    game.native_set_setting(m::ID, m::VERSION_KEY, &stored);
    panel.scroll = p::scroll_to_show(moved, panel.scroll, n, visible);
}
