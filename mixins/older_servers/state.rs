// The version the player picked on the Multiplayer screen, shared by the
// screen's button (the window's thread) and the connection (the network
// thread). Added to `horizon_client::net` as `older_servers_mod`.
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use horizon_protocol::adapter::ServerVersion;

/// The mod's id: its settings folder and its row in Options > Mods.
pub const ID: &str = "older_servers";
pub const NAME: &str = "Older Servers";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Join 1.21.10, 1.21.11, 26.1 and 26.2 servers without HORIZON_OLDER_SERVERS, \
and pick the version on the Multiplayer screen (top left), as ViaFabricPlus does. Unofficial.";

/// The choices, in the button's order: 0 is Auto (by the server's status ping),
/// then every version Horizon's adapter speaks, newest first.
pub fn choices() -> Vec<Option<ServerVersion>> {
    std::iter::once(None)
        .chain(ServerVersion::ALL.into_iter().map(Some))
        .collect()
}

static CHOICE: AtomicU8 = AtomicU8::new(0);
static LOADED: AtomicBool = AtomicBool::new(false);
static ON: AtomicBool = AtomicBool::new(true);

/// The picked version: None for Auto.
pub fn choice() -> Option<ServerVersion> {
    choices()
        .get(CHOICE.load(Ordering::Relaxed) as usize)
        .copied()
        .flatten()
}

/// Steps the choice forward (or back) and returns the value to store.
pub fn step(forward: bool) -> String {
    let n = choices().len() as u8;
    let at = CHOICE.load(Ordering::Relaxed) % n;
    let next = if forward { (at + 1) % n } else { (at + n - 1) % n };
    CHOICE.store(next, Ordering::Relaxed);
    stored()
}

/// The stored form: the version's name, or `auto`.
pub fn stored() -> String {
    choice().map_or_else(|| "auto".into(), |v| v.name().to_owned())
}

/// Reads the stored choice once (an unknown name is Auto).
pub fn load(stored: Option<String>) {
    if LOADED.swap(true, Ordering::Relaxed) {
        return;
    }
    let at = stored
        .and_then(|s| {
            choices()
                .iter()
                .position(|c| c.map_or("auto", |v| v.name()) == s.trim())
        })
        .unwrap_or(0);
    CHOICE.store(at as u8, Ordering::Relaxed);
}

/// Whether the mod is on in Options > Mods (off: Horizon's own behaviour).
pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// The button's text.
pub fn label() -> String {
    match choice() {
        None => "Sürüm: Otomatik".into(),
        Some(v) => format!("Sürüm: {}", v.name()),
    }
}

/// The button's rectangle on the Multiplayer screen: top left, as ViaFabricPlus's.
pub fn button(text_width: f32) -> [f32; 4] {
    [5.0, 5.0, (text_width + 16.0).max(98.0), 20.0]
}

pub fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}
