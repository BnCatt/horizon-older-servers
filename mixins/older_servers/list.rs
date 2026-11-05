// The mod's row in Options > Mods.
fn older_servers_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::net::older_servers_mod as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
