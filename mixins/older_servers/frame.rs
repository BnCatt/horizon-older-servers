// Her kare: ayarlar bir kez okunur, mod Options > Mods'taki anahtarını izler ve
// dünyadan çıkınca son bağlantının sürümü unutulur.
fn older_servers_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::net::older_servers_mod as m;
    m::set_on(game.native_on(m::ID));
    m::load(&|key: &str| game.native_setting(m::ID, key));
    m::track_world(game.native_player().is_some());
}
