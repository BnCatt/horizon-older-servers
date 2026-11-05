# Older Servers (Horizon native mod)

A native mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3 in
Rust) that makes its built-in older-servers adapter usable without environment variables, and adds a
ViaFabricPlus-style version picker.

- **Join 1.21.10, 1.21.11, 26.1 and 26.2 servers** without setting `HORIZON_OLDER_SERVERS=1`.
- **Version button, top left of the Multiplayer screen**: `Sürüm: Otomatik` (Auto: Horizon pings the
  server and picks its version), or a fixed version. Left click: next, right click: previous. The choice
  is saved in `horizon-mods/.horizon-mod-data/older_servers/version`.
- **1.21.11 item fix**: 1.21.11 sends `instrument`, `provides_trim_material`, `damage_type`,
  `chicken/variant` and `zombie_nautilus/variant` as EitherHolders; the adapter read them as 26.3's
  plain holders and dropped the packet (goat horns, trim materials, eggs…). Upstream:
  horizon-rust/horizon#9752.
- Shows up in Options > Mods as "Older Servers", where it can be turned off.

## Install

```sh
python tools/package.py --install     # target/older_servers-1.0.0.hnmod → horizon-native-mods/
```

Then build the game with its native mods in the Horizon launcher.

## Pinned

It changes functions that are not native hook points (`horizon_client_lib::net::older_servers_enabled`,
`::older_server`, `horizon_protocol::item::older_layout`), so it is pinned to Horizon Nightly
2026-11-05 (`f319701`). A newer Nightly may move those functions: re-check them and move the pin in
`compatibility.json` and `mixins/older_servers/horizon-mixin.toml`. Once the item fix lands upstream,
`either.rs` can go.

## Tested

Plans with 7 other mods and builds in the launcher (dev and release) on `f319701`; on the Multiplayer
screen the button shows and steps through the versions, and the choice is saved. The adapter itself and
the item fix were tested live against Paper 1.21.11 (see horizon-rust/horizon#9752).

Not affiliated with Mojang, Microsoft, the Horizon project or ViaFabricPlus.

## License

MIT, see [LICENSE](LICENSE).
