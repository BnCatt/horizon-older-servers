// ViaFabricPlus's F3 group: `[ViaFabricPlus] <version>`, and on a server
// `P: <steps> C: <client protocol> S: <server protocol>`.
fn older_servers_debug(game: &crate::game::Game<'_>, d: &mut Displayer) {
    use horizon_client::net::older_servers_mod as m;
    if !m::on() || !m::settings().debug_line || game.native_player().is_none() {
        return;
    }
    d.group("viafabricplus", m::vfp_debug(m::connected()));
}
