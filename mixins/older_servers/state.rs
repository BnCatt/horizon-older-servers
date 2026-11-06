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
pub const VERSION: &str = "1.1.0";
pub const DESCRIPTION: &str = "Join 1.21.10, 1.21.11, 26.1 and 26.2 servers without HORIZON_OLDER_SERVERS: \
a version list on the Multiplayer screen, a version per server, version tags in the server list and an \
F3 line, in the manner of ViaFabricPlus. Unofficial.";

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
    CHOICE.store(choice_from(get(VERSION_KEY).as_deref()) as u8, Ordering::Relaxed);
    *SETTINGS.lock().unwrap() = Settings::load(get);
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

/// Butonun durduğu köşe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    const ALL: [Self; 4] = [Self::TopLeft, Self::TopRight, Self::BottomLeft, Self::BottomRight];

    pub fn key(self) -> &'static str {
        match self {
            Self::TopLeft => "top_left",
            Self::TopRight => "top_right",
            Self::BottomLeft => "bottom_left",
            Self::BottomRight => "bottom_right",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Sol üst",
            Self::TopRight => "Sağ üst",
            Self::BottomLeft => "Sol alt",
            Self::BottomRight => "Sağ alt",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.key() == s.trim())
    }
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

/// Modun ayarları (Options > Mods > Older Servers).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Seçim yokken sunucunun sürümü ping ile bulunur; kapalıyken 26.3 olarak bağlanılır.
    pub auto_detect: bool,
    pub button_corner: Corner,
    pub hide_button: bool,
    /// Sunucu listesinde sürüm etiketi.
    pub list_tags: bool,
    /// F3 ekranında bağlanılan sürüm satırı.
    pub debug_line: bool,
}

impl Settings {
    pub const DEFAULT: Self = Self {
        auto_detect: true,
        button_corner: Corner::TopLeft,
        hide_button: false,
        list_tags: true,
        debug_line: true,
    };

    /// Ayar dosyalarından okur; olmayan ya da okunamayan her ayar varsayılanında kalır.
    pub fn load(get: &dyn Fn(&str) -> Option<String>) -> Self {
        let flag = |key: &str, default: bool| match get(key).as_deref().map(str::trim) {
            Some("true") => true,
            Some("false") => false,
            _ => default,
        };
        let d = Self::DEFAULT;
        Self {
            auto_detect: flag("auto_detect", d.auto_detect),
            button_corner: get("button_corner")
                .as_deref()
                .and_then(Corner::parse)
                .unwrap_or(d.button_corner),
            hide_button: flag("hide_button", d.hide_button),
            list_tags: flag("list_tags", d.list_tags),
            debug_line: flag("debug_line", d.debug_line),
        }
    }

    /// Saklanacak anahtar/değer çiftleri.
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            ("auto_detect", self.auto_detect.to_string()),
            ("button_corner", self.button_corner.key().to_owned()),
            ("hide_button", self.hide_button.to_string()),
            ("list_tags", self.list_tags.to_string()),
            ("debug_line", self.debug_line.to_string()),
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

fn on_off(b: bool) -> &'static str {
    if b { "AÇIK" } else { "KAPALI" }
}

/// Ayar sayfasının butonları, sırasıyla.
pub fn settings_labels() -> Vec<String> {
    let s = settings();
    vec![
        format!("Otomatik algılama: {}", on_off(s.auto_detect)),
        format!("Buton köşesi: {}", s.button_corner.label()),
        format!("Butonu gizle: {}", on_off(s.hide_button)),
        format!("Listede sürüm etiketi: {}", on_off(s.list_tags)),
        format!("F3 satırı: {}", on_off(s.debug_line)),
        label(),
    ]
}

/// Ayar sayfasında `i`. butona basıldı: saklanacak anahtar/değer çiftleri.
pub fn settings_click(i: usize) -> Vec<(&'static str, String)> {
    if i == 5 {
        return vec![(VERSION_KEY, step(true))];
    }
    let mut s = SETTINGS.lock().unwrap();
    match i {
        0 => s.auto_detect = !s.auto_detect,
        1 => s.button_corner = s.button_corner.next(),
        2 => s.hide_button = !s.hide_button,
        3 => s.list_tags = !s.list_tags,
        4 => s.debug_line = !s.debug_line,
        _ => return Vec::new(),
    }
    s.stored()
}

// ---------------------------------------------------------------- buton yerleşimi

/// Multiplayer ekranındaki butonun dikdörtgeni, ayarlardaki köşede (ekran `size`).
pub fn button_at(corner: Corner, size: [f32; 2], text_width: f32) -> [f32; 4] {
    let w = (text_width + 16.0).max(98.0);
    let x = match corner {
        Corner::TopLeft | Corner::BottomLeft => 5.0,
        Corner::TopRight | Corner::BottomRight => size[0] - w - 5.0,
    };
    let y = match corner {
        Corner::TopLeft | Corner::TopRight => 5.0,
        Corner::BottomLeft | Corner::BottomRight => size[1] - 25.0,
    };
    [x, y, w, 20.0]
}

/// Multiplayer ekranındaki butonun dikdörtgeni, şu anki ayarlarla.
pub fn button(size: [f32; 2], text_width: f32) -> [f32; 4] {
    button_at(settings().button_corner, size, text_width)
}

/// Edit/Add Server ekranında sürüm butonu: adres kutusunun (y 106, 200 geniş, ortalı)
/// sağında; sığmazsa solunda; o da olmazsa sağ üstte.
pub fn edit_button(size: [f32; 2], text_width: f32) -> [f32; 4] {
    let w = (text_width + 16.0).max(70.0);
    let field = (size[0] / 2.0).floor() - 100.0;
    if field + 200.0 + 4.0 + w <= size[0] - 2.0 {
        [field + 204.0, 106.0, w, 20.0]
    } else if field - 4.0 - w >= 2.0 {
        [field - 4.0 - w, 106.0, w, 20.0]
    } else {
        [size[0] - w - 5.0, 5.0, w, 20.0]
    }
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
            button_corner: Corner::BottomRight,
            hide_button: true,
            list_tags: false,
            debug_line: false,
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
    fn the_buttons_sit_in_their_corner() {
        assert_eq!(button_at(Corner::TopLeft, [427.0, 240.0], 50.0), [5.0, 5.0, 98.0, 20.0]);
        assert_eq!(button_at(Corner::BottomRight, [427.0, 240.0], 50.0), [324.0, 215.0, 98.0, 20.0]);
        // 427 genişlikte adres kutusu 113..313: buton sağında.
        assert_eq!(edit_button([427.0, 240.0], 60.0), [317.0, 106.0, 76.0, 20.0]);
        // 480 genişlikte de sağında; 320'de iki yana da sığmaz: sağ üst.
        assert_eq!(edit_button([480.0, 240.0], 60.0)[0], 344.0);
        assert_eq!(edit_button([320.0, 240.0], 60.0), [239.0, 5.0, 76.0, 20.0]);
    }
}
