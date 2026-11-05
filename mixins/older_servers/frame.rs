// Each frame: the stored choice is read once, and the mod follows its switch
// in Options > Mods.
fn older_servers_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::net::older_servers_mod as m;
    m::set_on(game.native_on(m::ID));
    m::load(game.native_setting(m::ID, "version"));
}
