// F3 ekranında bağlanılan sürüm: `Older Servers: 1.21.11 (774), auto`.
fn older_servers_debug(game: &crate::game::Game<'_>, d: &mut Displayer) {
    use horizon_client::net::older_servers_mod as m;
    if !m::on() || !m::settings().debug_line || game.native_player().is_none() {
        return;
    }
    if let Some(line) = m::debug_line(m::connected()) {
        d.line(line);
    }
}
