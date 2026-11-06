// Older Servers'ın paylaşılan durumu: genel sürüm seçimi, ayarlar, sunucu başına
// sabitlenen sürümler, sunucu listesi ping'lerinden okunan protokoller ve bağlı
// olunan sürüm. Ekran hook'ları (pencere iş parçacığı) ile bağlantı kodu (ağ iş
// parçacığı) aynı statik'leri okur. `horizon_client::net` içine
// `older_servers_mod` olarak eklenir.
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use horizon_protocol::adapter::ServerVersion;

/// Modun kimliği: ayar klasörü ve Options > Mods satırı.
pub const ID: &str = "older_servers";
pub const NAME: &str = "Older Servers";
pub const VERSION: &str = "2.0.0";
pub const DESCRIPTION: &str = "Join 1.21.10, 1.21.11, 26.1 and 26.2 servers without HORIZON_OLDER_SERVERS: \
ViaFabricPlus's screens (protocol selection with tabs, search and favourites, Force Version per server, \
settings), the server version in the ping tooltip and its F3 lines. Not affiliated with ViaFabricPlus.";

/// Sunucu başına sürümlerin tutulduğu ayar dosyası.
pub const SERVERS_KEY: &str = "servers.json";
/// Genel seçimin anahtarı (1.0.0'dan beri: `auto` ya da sürüm adı).
pub const VERSION_KEY: &str = "version";

// ---------------------------------------------------------------- seçim

/// Listedeki seçenekler: 0 "seçim yok" (otomatik algılama ya da 26.3), sonra
/// adapter'ın konuştuğu her sürüm, en yeni üstte (`ServerVersion::ALL` sırası).
pub fn choices() -> Vec<Option<ServerVersion>> {
    std::iter::once(None)
        .chain(ServerVersion::ALL.into_iter().map(Some))
        .collect()
}

static CHOICE: AtomicU8 = AtomicU8::new(0);
static LOADED: AtomicBool = AtomicBool::new(false);
static ON: AtomicBool = AtomicBool::new(true);

/// Seçili satırın sırası (`choices()` içinde).
pub fn choice_index() -> usize {
    let n = choices().len();
    (CHOICE.load(Ordering::Relaxed) as usize).min(n - 1)
}

/// Genel seçim: None = seçim yok.
pub fn choice() -> Option<ServerVersion> {
    choices().get(choice_index()).copied().flatten()
}

/// `i`. seçeneği seçer, saklanacak değeri döndürür.
pub fn set_choice(i: usize) -> String {
    let n = choices().len();
    CHOICE.store(i.min(n - 1) as u8, Ordering::Relaxed);
    stored()
}

/// Seçimi bir ileri (ya da geri) kaydırır, saklanacak değeri döndürür.
pub fn step(forward: bool) -> String {
    let n = choices().len();
    let at = choice_index();
    set_choice(if forward { (at + 1) % n } else { (at + n - 1) % n })
}

/// Saklanan biçim: sürümün adı ya da `auto`.
pub fn stored() -> String {
    stored_name(choice())
}

fn stored_name(v: Option<ServerVersion>) -> String {
    v.map_or_else(|| "auto".into(), |v| v.name().to_owned())
}

/// Saklanan bir adı sürüme çevirir (`auto`/bilinmeyen: None).
pub fn parse_version(s: &str) -> Option<ServerVersion> {
    let s = s.trim();
    ServerVersion::ALL.into_iter().find(|v| v.name() == s)
}

fn choice_from(stored: Option<&str>) -> usize {
    stored
        .and_then(parse_version)
        .and_then(|v| choices().iter().position(|c| *c == Some(v)))
        .unwrap_or(0)
}

/// Ayarları bir kez okur: genel seçim, ayarlar ve sunucu başına sürümler.
pub fn load(get: &dyn Fn(&str) -> Option<String>) {
    if LOADED.swap(true, Ordering::Relaxed) {
        return;
    }
    let settings = Settings::load(get);
    let version = if settings.save_selected { get(VERSION_KEY) } else { None };
    CHOICE.store(choice_from(version.as_deref()) as u8, Ordering::Relaxed);
    *SETTINGS.lock().unwrap() = settings;
    *FAVORITES.lock().unwrap() = get(FAVORITES_KEY)
        .map(|s| s.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default();
    load_forced(get(SERVERS_KEY).as_deref());
}

/// Options > Mods'taki anahtar (kapalıyken Horizon'un kendi davranışı).
pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Listede bir seçeneğin adı.
pub fn choice_label(item: Option<ServerVersion>, auto_detect: bool) -> String {
    match item {
        None if auto_detect => "Otomatik (sunucuya göre)".into(),
        None => format!("Varsayılan ({})", ServerVersion::Current.name()),
        Some(v) => v.name().to_owned(),
    }
}

/// Multiplayer ekranındaki butonun yazısı.
pub fn label() -> String {
    match choice() {
        None if settings().auto_detect => "Sürüm: Otomatik".into(),
        None => format!("Sürüm: {}", ServerVersion::Current.name()),
        Some(v) => format!("Sürüm: {}", v.name()),
    }
}

// ---------------------------------------------------------------- ayarlar

use super::older_servers_vfp::Orientation;

/// Modun ayarları: ViaFabricPlus'ın General grubundan buraya uyanlar, ve
/// Auto Detect (seçili sürüm "Auto Detect" mi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Seçim yokken sunucunun sürümü ping ile bulunur (Auto Detect).
    pub auto_detect: bool,
    /// F3 satırları.
    pub debug_line: bool,
    pub mp_button: Orientation,
    pub add_button: Orientation,
    pub direct_button: Orientation,
    /// Seçilen sürüm saklanır; kapalıyken her açılışta Auto Detect.
    pub save_selected: bool,
    /// Sunucu listesinin ping ipucunda hedef ve sunucu sürümü.
    pub show_advertised: bool,
}

impl Settings {
    pub const DEFAULT: Self = Self {
        auto_detect: true,
        debug_line: true,
        mp_button: Orientation::RightTop,
        add_button: Orientation::RightTop,
        direct_button: Orientation::RightTop,
        save_selected: true,
        show_advertised: true,
    };

    /// Ayar dosyalarından okur; olmayan ya da okunamayan her ayar varsayılanında kalır.
    pub fn load(get: &dyn Fn(&str) -> Option<String>) -> Self {
        let flag = |key: &str, default: bool| match get(key).as_deref().map(str::trim) {
            Some("true") => true,
            Some("false") => false,
            _ => default,
        };
        let corner = |key: &str, default: Orientation| {
            get(key).as_deref().and_then(Orientation::parse).unwrap_or(default)
        };
        let d = Self::DEFAULT;
        Self {
            auto_detect: flag("auto_detect", d.auto_detect),
            debug_line: flag("debug_line", d.debug_line),
            mp_button: corner("mp_button", d.mp_button),
            add_button: corner("add_button", d.add_button),
            direct_button: corner("direct_button", d.direct_button),
            save_selected: flag("save_selected", d.save_selected),
            show_advertised: flag("show_advertised", d.show_advertised),
        }
    }

    /// Saklanacak anahtar/değer çiftleri.
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            ("auto_detect", self.auto_detect.to_string()),
            ("debug_line", self.debug_line.to_string()),
            ("mp_button", self.mp_button.key().to_owned()),
            ("add_button", self.add_button.key().to_owned()),
            ("direct_button", self.direct_button.key().to_owned()),
            ("save_selected", self.save_selected.to_string()),
            ("show_advertised", self.show_advertised.to_string()),
        ]
    }
}

static SETTINGS: Mutex<Settings> = Mutex::new(Settings::DEFAULT);

pub fn settings() -> Settings {
    *SETTINGS.lock().unwrap()
}

/// "Otomatik algılama"yı çevirir, saklanacak çiftleri döndürür.
pub fn toggle_auto() -> Vec<(&'static str, String)> {
    let mut s = SETTINGS.lock().unwrap();
    s.auto_detect = !s.auto_detect;
    s.stored()
}

/// ViaFabricPlus'ın ayar ekranında `i`. satır (`older_servers_vfp::SETTINGS`).
pub fn setting_click(i: usize) -> Vec<(&'static str, String)> {
    let mut s = SETTINGS.lock().unwrap();
    match i {
        0 => s.mp_button = s.mp_button.next(),
        1 => s.add_button = s.add_button.next(),
        2 => s.direct_button = s.direct_button.next(),
        3 => s.save_selected = !s.save_selected,
        4 => s.show_advertised = !s.show_advertised,
        _ => return Vec::new(),
    }
    s.stored()
}

/// Seçilen sürüm (ViaFabricPlus ekranında bir satır): None = Auto Detect.
/// Saklanacak çiftler; "Save selected protocol version" kapalıyken sürüm saklanmaz.
pub fn set_target(v: Option<ServerVersion>) -> Vec<(&'static str, String)> {
    let index = v.and_then(|v| choices().iter().position(|c| *c == Some(v))).unwrap_or(0);
    let version = set_choice(index);
    let mut s = SETTINGS.lock().unwrap();
    s.auto_detect = v.is_none() || s.auto_detect;
    let mut out = s.stored();
    if s.save_selected {
        out.push((VERSION_KEY, version));
    }
    out
}

fn on_off(b: bool) -> &'static str {
    if b { "On" } else { "Off" }
}

/// Options > Mods > Older Servers sayfasının butonları, sırasıyla.
pub fn settings_labels() -> Vec<String> {
    let s = settings();
    vec![
        format!("Multiplayer button: {}", s.mp_button.label()),
        format!("Add server button: {}", s.add_button.label()),
        format!("Direct connect button: {}", s.direct_button.label()),
        format!("Save selected protocol version: {}", on_off(s.save_selected)),
        format!("Show advertised/server version: {}", on_off(s.show_advertised)),
        format!("F3 lines: {}", on_off(s.debug_line)),
    ]
}

/// Ayar sayfasında `i`. butona basıldı: saklanacak anahtar/değer çiftleri.
pub fn settings_click(i: usize) -> Vec<(&'static str, String)> {
    if i == 5 {
        let mut s = SETTINGS.lock().unwrap();
        s.debug_line = !s.debug_line;
        return s.stored();
    }
    setting_click(i)
}

// ---------------------------------------------------------------- favoriler

pub const FAVORITES_KEY: &str = "favorites";
static FAVORITES: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn is_favorite(name: &str) -> bool {
    FAVORITES.lock().unwrap().iter().any(|f| f == name)
}

/// Yıldıza tıklandı: saklanacak liste (satır başına bir ad).
pub fn toggle_favorite(name: &str) -> String {
    let mut f = FAVORITES.lock().unwrap();
    if let Some(i) = f.iter().position(|x| x == name) {
        f.remove(i);
    } else {
        f.push(name.to_owned());
    }
    f.join("\n")
}

pub fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}

// ---------------------------------------------------------------- sunucu başına sürüm

/// Bir adresin tek yazımı: küçük harf, sondaki nokta atılır, port yoksa `:25565`;
/// IPv6 köşeli parantezle yazılır.
pub fn normalize(address: &str) -> String {
    let a = address.trim().to_ascii_lowercase();
    let (host, port) = if let Some(rest) = a.strip_prefix('[') {
        // [v6] ya da [v6]:port
        match rest.split_once(']') {
            Some((h, tail)) => (format!("[{h}]"), tail.strip_prefix(':').map(str::to_owned)),
            None => (format!("[{rest}]"), None),
        }
    } else if a.matches(':').count() > 1 {
        // Parantezsiz IPv6: portu yok sayılır.
        (format!("[{a}]"), None)
    } else {
        match a.rsplit_once(':') {
            Some((h, p)) => (h.to_owned(), Some(p.to_owned())),
            None => (a.clone(), None),
        }
    };
    let host = if host.starts_with('[') {
        host
    } else {
        host.trim_end_matches('.').to_owned()
    };
    let port = port
        .filter(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or_else(|| "25565".into());
    format!("{host}:{port}")
}

static FORCED: Mutex<BTreeMap<String, ServerVersion>> = Mutex::new(BTreeMap::new());

/// `servers.json`'u okur (`{"<normalize>": "<ad>"}`); bozuk dosya boş harita sayılır.
pub fn load_forced(json: Option<&str>) {
    *FORCED.lock().unwrap() = parse_forced(json);
}

/// `servers.json`'un içeriği; bilinmeyen sürüm adları atlanır.
pub fn parse_forced(json: Option<&str>) -> BTreeMap<String, ServerVersion> {
    let map: BTreeMap<String, String> = json
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_default();
    map.into_iter()
        .filter_map(|(address, name)| Some((normalize(&address), parse_version(&name)?)))
        .collect()
}

/// Sunucuya sabitlenen sürüm.
pub fn forced(address: &str) -> Option<ServerVersion> {
    FORCED.lock().unwrap().get(&normalize(address)).copied()
}

/// Sunucunun sürümünü sabitler (None: kaldırır), saklanacak JSON'u döndürür.
pub fn set_forced(address: &str, v: Option<ServerVersion>) -> String {
    let mut forced = FORCED.lock().unwrap();
    let key = normalize(address);
    match v {
        Some(v) => {
            forced.insert(key, v);
        }
        None => {
            forced.remove(&key);
        }
    }
    let map: BTreeMap<&str, &str> = forced.iter().map(|(k, v)| (k.as_str(), v.name())).collect();
    serde_json::to_string(&map).unwrap_or_else(|_| "{}".into())
}

/// Edit/Add Server butonu: Varsayılan → en yeni → … → en eski → Varsayılan.
pub fn cycle_forced(address: &str, forward: bool) -> String {
    let order = choices();
    let at = order.iter().position(|c| *c == forced(address)).unwrap_or(0);
    let n = order.len();
    let next = if forward { (at + 1) % n } else { (at + n - 1) % n };
    set_forced(address, order[next])
}

/// Edit/Add Server butonunun yazısı.
pub fn server_label(address: &str) -> String {
    match forced(address) {
        None => "Sürüm: Varsayılan".into(),
        Some(v) => format!("Sürüm: {}", v.name()),
    }
}

// ---------------------------------------------------------------- bağlanırken seçim

/// Bağlanılan sürüm nereden geldi (F3 satırı).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Sunucuya sabitlenmiş.
    Server,
    /// Listeden seçilmiş.
    Picked,
    /// Sunucunun ping'inden.
    Detected,
    /// Seçim yok, otomatik algılama kapalı: 26.3.
    Default,
}

impl Source {
    fn word(self) -> &'static str {
        match self {
            Self::Server => "per server",
            Self::Picked => "picked",
            Self::Detected => "auto",
            Self::Default => "default",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Use(ServerVersion, Source),
    /// Sunucuya ping atılıp sürümü bulunacak.
    Detect,
}

/// Öncelik: sunucuya sabit > genel seçim > otomatik algılama (kapalıysa 26.3).
pub fn decide(forced: Option<ServerVersion>, choice: Option<ServerVersion>, auto_detect: bool) -> Decision {
    match (forced, choice) {
        (Some(v), _) => Decision::Use(v, Source::Server),
        (None, Some(v)) => Decision::Use(v, Source::Picked),
        (None, None) if auto_detect => Decision::Detect,
        (None, None) => Decision::Use(ServerVersion::Current, Source::Default),
    }
}

/// Ping'in bildirdiği protokolün sürümü; bilinmeyen ya da ping yoksa 26.3.
pub fn detected(protocol: Option<i32>) -> ServerVersion {
    protocol
        .and_then(ServerVersion::from_protocol)
        .unwrap_or(ServerVersion::Current)
}

static CONNECTED: Mutex<Option<(ServerVersion, Source)>> = Mutex::new(None);
static IN_WORLD: AtomicBool = AtomicBool::new(false);

/// Son bağlantının sürümü (F3 satırı için).
pub fn set_connected(v: Option<(ServerVersion, Source)>) {
    *CONNECTED.lock().unwrap() = v;
}
pub fn connected() -> Option<(ServerVersion, Source)> {
    *CONNECTED.lock().unwrap()
}

/// Her kare: dünyadan çıkınca bağlantı bilgisi silinir (sonraki tek oyunculu
/// dünyada eski sunucunun sürümü görünmesin).
pub fn track_world(in_world: bool) {
    if IN_WORLD.swap(in_world, Ordering::Relaxed) && !in_world {
        set_connected(None);
    }
}

/// ViaFabricPlus'ın F3 grubu: `[ViaFabricPlus] <sürüm>`, bağlıyken
/// `P: <adım> C: <istemci protokolü> S: <sunucu protokolü>`.
pub fn vfp_debug(connected: Option<(ServerVersion, Source)>) -> Vec<String> {
    let mut lines = vec![format!("\u{a7}f[\u{a7}9ViaFabricPlus\u{a7}f]\u{a7}r {VERSION}")];
    if let Some((v, _)) = connected {
        let steps = ServerVersion::ALL.iter().position(|a| *a == v).unwrap_or(0);
        lines.push(format!(
            "P: {} C: {} S: {}",
            2 + steps,
            ServerVersion::Current.protocol(),
            v.protocol()
        ));
    }
    lines
}

/// Bir sunucuya hangi sürümle bağlanılacağı (ping ipucundaki "Target version").
pub fn target_for(address: &str) -> ServerVersion {
    match decide(forced(address), choice(), settings().auto_detect) {
        Decision::Use(v, _) => v,
        Decision::Detect => detected(pinged(address)),
    }
}

/// F3 satırı: `Older Servers: 1.21.11 (774), auto`.
pub fn debug_line(connected: Option<(ServerVersion, Source)>) -> Option<String> {
    let (v, source) = connected?;
    Some(format!("{NAME}: {} ({}), {}", v.name(), v.protocol(), source.word()))
}

// ---------------------------------------------------------------- sunucu listesi etiketi

static PINGED: Mutex<BTreeMap<String, i32>> = Mutex::new(BTreeMap::new());

/// Sunucu listesinin ping'inden okunan protokol.
pub fn record_ping(address: &str, protocol: i32) {
    PINGED.lock().unwrap().insert(normalize(address), protocol);
}
pub fn pinged(address: &str) -> Option<i32> {
    PINGED.lock().unwrap().get(&normalize(address)).copied()
}

pub const YELLOW: u32 = 0xFF_FF55;
pub const GREEN: u32 = 0x55_FF55;
pub const RED: u32 = 0xFF_5555;

/// Etiket: kullanılacak sürüm (`effective`) ve sunucunun bildirdiği protokol.
/// Sarı = çevrilecek, yeşil = 26.3, kırmızı = desteklenmiyor ya da sabitlenen
/// sürüm sunucununkiyle uyuşmuyor.
pub fn tag(effective: Option<ServerVersion>, pinged: Option<i32>) -> Option<(String, u32)> {
    let text = |v: ServerVersion| {
        if v == ServerVersion::Current {
            v.name().to_owned()
        } else {
            format!("→ {}", v.name())
        }
    };
    match (effective, pinged) {
        (Some(v), Some(p)) if p != v.protocol() => Some((text(v), RED)),
        (Some(v), _) if v == ServerVersion::Current => Some((text(v), GREEN)),
        (Some(v), _) => Some((text(v), YELLOW)),
        (None, Some(p)) => Some((format!("✕ {p}"), RED)),
        (None, None) => None,
    }
}

/// Bir sunucu satırının etiketi: sabit > seçim > ping'deki sürüm.
pub fn tag_for(address: &str, pinged_protocol: Option<i32>) -> Option<(String, u32)> {
    let s = settings();
    let effective = match decide(forced(address), choice(), s.auto_detect) {
        Decision::Use(v, _) => Some(v),
        Decision::Detect => pinged_protocol.and_then(ServerVersion::from_protocol),
    };
    tag(effective, pinged_protocol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: BTreeMap<String, String> =
            pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn an_old_file_with_only_a_version_keeps_the_defaults_and_the_version() {
        let get = files(&[("version", "1.21.11")]);
        assert_eq!(Settings::load(&get), Settings::DEFAULT);
        let at = choice_from(get("version").as_deref());
        assert_eq!(choices()[at], Some(ServerVersion::V1_21_11));
        assert_eq!(choice_from(Some("auto")), 0);
        assert_eq!(choice_from(Some("9.9")), 0);
    }

    #[test]
    fn settings_round_trip_through_their_files() {
        let s = Settings {
            auto_detect: false,
            debug_line: false,
            mp_button: Orientation::LeftBottom,
            add_button: Orientation::None,
            direct_button: Orientation::RightBottom,
            save_selected: false,
            show_advertised: false,
        };
        let stored: Vec<(String, String)> =
            s.stored().into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
        let get = move |k: &str| stored.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        assert_eq!(Settings::load(&get), s);
    }

    #[test]
    fn without_auto_detect_and_a_choice_it_speaks_26_3_without_a_ping() {
        assert_eq!(
            decide(None, None, false),
            Decision::Use(ServerVersion::Current, Source::Default)
        );
        assert_eq!(decide(None, None, true), Decision::Detect);
    }

    #[test]
    fn a_server_version_beats_the_choice_which_beats_auto_detect() {
        let v = ServerVersion::V1_21_11;
        let w = ServerVersion::V26_2;
        assert_eq!(decide(Some(v), Some(w), true), Decision::Use(v, Source::Server));
        assert_eq!(decide(None, Some(w), true), Decision::Use(w, Source::Picked));
        assert_eq!(decide(None, None, true), Decision::Detect);
        assert_eq!(detected(Some(774)), ServerVersion::V1_21_11);
        assert_eq!(detected(Some(5)), ServerVersion::Current);
        assert_eq!(detected(None), ServerVersion::Current);
    }

    #[test]
    fn addresses_have_one_spelling() {
        assert_eq!(normalize("Example.org"), "example.org:25565");
        assert_eq!(normalize("example.org.:25565"), "example.org:25565");
        assert_eq!(normalize(" EXAMPLE.org:25566 "), "example.org:25566");
        assert_eq!(normalize("[::1]"), "[::1]:25565");
        assert_eq!(normalize("[::1]:25570"), "[::1]:25570");
        assert_eq!(normalize("::1"), "[::1]:25565");
        assert_eq!(normalize("127.0.0.1:"), "127.0.0.1:25565");
    }

    #[test]
    fn a_version_set_for_one_spelling_is_found_by_another() {
        let json = set_forced("A.com", Some(ServerVersion::V1_21_11));
        assert_eq!(forced("a.com:25565"), Some(ServerVersion::V1_21_11));
        assert!(json.contains("\"a.com:25565\":\"1.21.11\""), "{json}");
        // Dosyadan geri okunur (farklı yazımla saklanmış olsa da).
        let read = parse_forced(Some(&json));
        assert_eq!(read.get("a.com:25565"), Some(&ServerVersion::V1_21_11));
        let read = parse_forced(Some(r#"{"B.org.":"1.21.10","c.net":"9.9"}"#));
        assert_eq!(read.get("b.org:25565"), Some(&ServerVersion::V1_21_10));
        assert_eq!(read.len(), 1);
        set_forced("a.com", None);
        assert_eq!(forced("a.com"), None);
        // Bozuk dosya: boş.
        assert!(parse_forced(Some("not json")).is_empty());
        assert!(parse_forced(None).is_empty());
    }

    #[test]
    fn the_edit_button_cycles_through_default_and_every_version() {
        let address = "cycle.test";
        set_forced(address, None);
        cycle_forced(address, true);
        assert_eq!(forced(address), Some(ServerVersion::ALL[0]));
        cycle_forced(address, false);
        assert_eq!(forced(address), None);
        cycle_forced(address, false);
        assert_eq!(forced(address), ServerVersion::ALL.last().copied());
        set_forced(address, None);
    }

    #[test]
    fn tags_are_yellow_green_or_red() {
        let v = ServerVersion::V1_21_11;
        assert_eq!(tag(Some(v), Some(774)), Some(("→ 1.21.11".into(), YELLOW)));
        assert_eq!(tag(Some(v), None), Some(("→ 1.21.11".into(), YELLOW)));
        let current = ServerVersion::Current;
        assert_eq!(
            tag(Some(current), Some(current.protocol())),
            Some((current.name().to_owned(), GREEN))
        );
        // Desteklenmeyen sunucu ve uyuşmayan sabit sürüm.
        assert_eq!(tag(None, Some(47)), Some(("✕ 47".into(), RED)));
        assert_eq!(tag(Some(v), Some(773)).map(|t| t.1), Some(RED));
        assert_eq!(tag(None, None), None);
    }

    #[test]
    fn the_debug_line_names_the_version_its_protocol_and_why() {
        assert_eq!(debug_line(None), None);
        assert_eq!(
            debug_line(Some((ServerVersion::V1_21_11, Source::Detected))).as_deref(),
            Some("Older Servers: 1.21.11 (774), auto")
        );
    }

    #[test]
    fn leaving_a_world_forgets_the_connection() {
        set_connected(Some((ServerVersion::V1_21_11, Source::Picked)));
        track_world(false); // bağlanırken: henüz dünyada değil
        assert!(connected().is_some());
        track_world(true);
        assert!(connected().is_some());
        track_world(false);
        assert_eq!(connected(), None);
    }

    #[test]
    fn favourites_toggle_and_store_one_per_line() {
        assert!(!is_favorite("1.21.11"));
        assert_eq!(toggle_favorite("1.21.11"), "1.21.11");
        assert_eq!(toggle_favorite("26.2"), "1.21.11
26.2");
        assert!(is_favorite("26.2"));
        assert_eq!(toggle_favorite("1.21.11"), "26.2");
        toggle_favorite("26.2");
    }
}
