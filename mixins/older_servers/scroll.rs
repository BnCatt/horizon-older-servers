// Panel açıkken tekerlek paneli kaydırır, alttaki sunucu listesini değil.
fn older_servers_scroll(
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    scroll: [f64; 2],
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
    panel.scroll = p::wheel(panel.scroll, scroll[1], n, visible);
}
