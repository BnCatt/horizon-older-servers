// Ayar sayfasında bir butona basıldı: ayar değişir ve kaydedilir.
fn older_servers_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use horizon_client::net::older_servers_mod as m;
    if mod_id != m::ID {
        return;
    }
    for (key, value) in m::settings_click(button) {
        game.native_set_setting(m::ID, key, &value);
    }
}
