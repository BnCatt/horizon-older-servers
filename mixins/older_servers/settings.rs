// Options > Mods > Older Servers sayfasının butonları.
fn older_servers_settings(mod_id: &str, buttons: &mut Vec<String>) {
    use horizon_client::net::older_servers_mod as m;
    if mod_id == m::ID {
        buttons.extend(m::settings_labels());
    }
}
