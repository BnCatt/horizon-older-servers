// ViaFabricPlus's screens for Horizon: the protocol selection screen (title,
// Back, search, category tabs, a list of every release with its icon, update
// name, release date and a favourite star, Server lists / Settings / Report
// issues), Force Version for a server, Settings and Report issues, and the
// 98x20 buttons on the Multiplayer, Add/Edit Server and Direct Connect screens.
// Written from ViaFabricPlus's behaviour and layout (its numbers: 260 rows of
// 32, tabs at 60, footer 30, accent 0x58A6FF); none of its code. Horizon
// doesn't let a mod open its own screen over the title menus, so these are
// drawn over the Multiplayer screen (`vfp_draw.rs`) and take all its input.
// This file: state, geometry and events. Added to `horizon_client::net` as
// `older_servers_vfp`.
use std::sync::Mutex;

use horizon_protocol::adapter::ServerVersion;

use super::older_servers_catalog::{RELEASES, Release};
use super::older_servers_mod as m;

pub const ACCENT: u32 = 0x58_A6FF;
pub const SELECTED_BACKGROUND: u32 = 0x4058_A6FF;
pub const SUBTITLE: u32 = 0xAAAAAA;
pub const GRAY: u32 = 0x808080;
pub const FAVORITE: u32 = 0xFFAA00;
pub const NO_FAVORITE: u32 = 0x555555;
pub const GREEN: u32 = 0x55FF55;
pub const RED: u32 = 0xFF5555;

pub const BUTTON_W: f32 = 98.0;
pub const SEARCH_TOP: f32 = 36.0;
pub const CONTENT_TOP: f32 = 60.0;
pub const TAB_H: f32 = 24.0;
pub const LIST_TOP: f32 = CONTENT_TOP + TAB_H;
pub const FOOTER_H: f32 = 30.0;
pub const SLOT_MARGIN: f32 = 3.0;
pub const ICON: f32 = 16.0;
pub const TEXT_OFFSET: f32 = SLOT_MARGIN + ICON + SLOT_MARGIN;
pub const STAR_AREA: f32 = TEXT_OFFSET;
pub const STAR: &str = "\u{2605}";
pub const TITLE: &str = "ViaFabricPlus";
pub const AUTO_NAME: &str = "Auto Detect (1.7+ servers)";
pub const AUTO_ICON: &str = "minecraft:spyglass";
pub const REPO: &str = "https://github.com/BnCatt/horizon-older-servers";

// ---------------------------------------------------------------- tabs

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Favorites,
    Modern,
    Legacy,
    Beta,
    Alpha,
    Classic,
    AprilFools,
}

impl Category {
    pub const TABS: [Category; 7] = [
        Category::Favorites,
        Category::Modern,
        Category::Legacy,
        Category::Beta,
        Category::Alpha,
        Category::Classic,
        Category::AprilFools,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Category::Favorites => "Favorites",
            Category::Modern => "Modern",
            Category::Legacy => "Legacy",
            Category::Beta => "Beta",
            Category::Alpha => "Alpha",
            Category::Classic => "Classic",
            Category::AprilFools => "April Fools",
        }
    }
}

pub const SETTINGS_TABS: [&str; 1] = ["General"];

// ---------------------------------------------------------------- button corners

/// ViaFabricPlus's `Orientation`: where a 98x20 button sits, or none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    None,
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
}

impl Orientation {
    const ALL: [Orientation; 5] = [
        Orientation::None,
        Orientation::LeftTop,
        Orientation::RightTop,
        Orientation::LeftBottom,
        Orientation::RightBottom,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Orientation::None => "none",
            Orientation::LeftTop => "left_top",
            Orientation::RightTop => "right_top",
            Orientation::LeftBottom => "left_bottom",
            Orientation::RightBottom => "right_bottom",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Orientation::None => "None",
            Orientation::LeftTop => "Left; Top",
            Orientation::RightTop => "Right; Top",
            Orientation::LeftBottom => "Left; Bottom",
            Orientation::RightBottom => "Right; Bottom",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|o| o.key() == s.trim())
    }
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|o| *o == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
    /// The button's rectangle on a `size` screen.
    pub fn rect(self, size: [f32; 2]) -> Option<[f32; 4]> {
        let [w, h] = size;
        let (x, y) = match self {
            Orientation::None => return None,
            Orientation::LeftTop => (5.0, 5.0),
            Orientation::RightTop => (w - BUTTON_W - 5.0, 5.0),
            Orientation::LeftBottom => (5.0, h - 20.0 - 5.0),
            Orientation::RightBottom => (w - BUTTON_W - 5.0, h - 20.0 - 5.0),
        };
        Some([x, y, BUTTON_W, 20.0])
    }
}

// ---------------------------------------------------------------- releases

/// The releases Horizon's adapter can join, by ViaFabricPlus's names.
pub fn supported(name: &str) -> Option<ServerVersion> {
    Some(match name {
        "26.3" => ServerVersion::Current,
        "26.2" => ServerVersion::V26_2,
        "26.1-26.1.2" => ServerVersion::V26_1,
        "1.21.11" => ServerVersion::V1_21_11,
        "1.21.9-1.21.10" => ServerVersion::V1_21_10,
        _ => return None,
    })
}

/// ViaFabricPlus's name of a version Horizon joins.
pub fn name_of(v: ServerVersion) -> &'static str {
    match v {
        ServerVersion::Current => "26.3",
        ServerVersion::V26_2 => "26.2",
        ServerVersion::V26_1 => "26.1-26.1.2",
        ServerVersion::V1_21_11 => "1.21.11",
        ServerVersion::V1_21_10 => "1.21.9-1.21.10",
    }
}

/// A row of a version list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Auto,
    Release(usize),
}

impl Slot {
    pub fn release(self) -> Option<&'static Release> {
        match self {
            Slot::Auto => None,
            Slot::Release(i) => RELEASES.get(i),
        }
    }
    pub fn name(self) -> &'static str {
        self.release().map_or(AUTO_NAME, |r| r.name)
    }
    pub fn icon(self) -> &'static str {
        self.release().map_or(AUTO_ICON, |r| r.icon)
    }
    pub fn version(self) -> Option<ServerVersion> {
        self.release().and_then(|r| supported(r.name))
    }
    /// Horizon can join it (Auto Detect too).
    pub fn joinable(self) -> bool {
        matches!(self, Slot::Auto) || self.version().is_some()
    }
}

// ---------------------------------------------------------------- pages

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// The protocol selection screen (the main one).
    Protocols,
    /// Force Version for the server being added or edited.
    Force,
    Settings,
    Report,
}

impl Page {
    fn list(self) -> Option<ListKind> {
        match self {
            Page::Protocols | Page::Force => Some(ListKind::Protocols),
            Page::Settings => Some(ListKind::Settings),
            Page::Report => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Protocols,
    Settings,
}

impl ListKind {
    pub fn item_height(self) -> f32 {
        match self {
            ListKind::Protocols => 32.0,
            // (font.lineHeight + 2) * 2
            ListKind::Settings => 22.0,
        }
    }
    pub fn row_width(self, w: f32) -> f32 {
        match self {
            ListKind::Protocols => 260.0_f32.min(w - 20.0),
            ListKind::Settings => 360.0_f32.min(w - 20.0),
        }
    }
    pub fn bottom_margin(self) -> f32 {
        match self {
            ListKind::Protocols => FOOTER_H,
            ListKind::Settings => 5.0,
        }
    }
    pub fn tabs(self) -> Vec<&'static str> {
        match self {
            ListKind::Protocols => Category::TABS.iter().map(|c| c.title()).collect(),
            ListKind::Settings => SETTINGS_TABS.to_vec(),
        }
    }
}

/// The settings page's rows (ViaFabricPlus's General group, the ones that
/// mean something here).
pub const SETTINGS: [&str; 5] = [
    "Multiplayer screen button orientation",
    "Add server screen button orientation",
    "Direct connect screen button orientation",
    "Save selected protocol version",
    "Show advertised/server version in Multiplayer",
];

/// A setting's value text and colour.
pub fn setting_value(i: usize) -> (&'static str, u32) {
    let s = m::settings();
    let flag = |b: bool| if b { ("On", GREEN) } else { ("Off", RED) };
    match i {
        0 => (s.mp_button.label(), 0xFFFFFF),
        1 => (s.add_button.label(), 0xFFFFFF),
        2 => (s.direct_button.label(), 0xFFFFFF),
        3 => flag(s.save_selected),
        _ => flag(s.show_advertised),
    }
}

/// Report issues' buttons: label, and whether it does anything here.
pub const REPORT: [(&str, bool); 4] = [
    ("Report a bug on GitHub", true),
    ("Request a feature on GitHub", true),
    ("Create a ViaVersion dump", false),
    ("Open logs folder", true),
];

// ---------------------------------------------------------------- state

pub struct Ui {
    /// Open pages, the shown one last; empty: closed.
    pub stack: Vec<Page>,
    /// The open tab of each list kind, and each tab's scroll.
    pub tab: [usize; 2],
    pub scroll: [[crate::ui::Scroll; 8]; 2],
    /// The search box (one per open screen, as ViaFabricPlus builds it in `init`).
    pub search: crate::ui::EditBox,
    pub search_scroll: crate::ui::Scroll,
    /// The address of the server Force Version is for.
    pub force: Option<String>,
}

impl Ui {
    fn new() -> Self {
        let mut search = crate::ui::EditBox::new(0.0, 0.0, 0.0, 20.0);
        search.max_length = 32;
        Self {
            stack: Vec::new(),
            tab: [1, 0],
            scroll: [[crate::ui::Scroll::default(); 8]; 2],
            search,
            search_scroll: crate::ui::Scroll::default(),
            force: None,
        }
    }
    pub fn page(&self) -> Option<Page> {
        self.stack.last().copied()
    }
}

pub static UI: Mutex<Option<Ui>> = Mutex::new(None);

/// Runs `f` on the state (made on first use).
pub fn with<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
    let mut guard = UI.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Ui::new))
}

pub fn page() -> Option<Page> {
    with(|ui| ui.page())
}

/// A page opens over the current one. The Protocols and Settings screens
/// start on the tab holding the selection, with an empty search.
pub fn open(page: Page, force: Option<String>) {
    with(|ui| {
        if ui.stack.is_empty() || matches!(page, Page::Protocols | Page::Force | Page::Settings) {
            ui.search.set_value("");
            ui.search.focused = false;
        }
        if page == Page::Force {
            ui.force = force;
        }
        if let Some(kind) = page.list() {
            let k = kind as usize;
            if kind == ListKind::Protocols {
                ui.tab[k] = initial_tab(page, ui.force.as_deref());
            }
        }
        ui.stack.push(page);
    })
}

/// Back (Escape, the Back button, the popup's cross): to the page under.
pub fn back() {
    with(|ui| {
        ui.stack.pop();
        ui.search.set_value("");
        ui.search.focused = false;
    })
}

/// Everything closes (another screen opened).
pub fn close() {
    with(|ui| ui.stack.clear())
}

// ---------------------------------------------------------------- selection

/// The version picked on the main screen: Auto Detect or a release.
pub fn target() -> Slot {
    let s = m::settings();
    let picked = match m::choice() {
        Some(v) => v,
        None if s.auto_detect => return Slot::Auto,
        None => ServerVersion::Current,
    };
    release_slot(name_of(picked)).unwrap_or(Slot::Auto)
}

pub fn release_slot(name: &str) -> Option<Slot> {
    RELEASES.iter().position(|r| r.name == name).map(Slot::Release)
}

pub fn selected(page: Page, slot: Slot, force: Option<&str>) -> bool {
    match page {
        Page::Force => match (slot, force.and_then(m::forced)) {
            (Slot::Release(_), Some(v)) => slot.version() == Some(v),
            _ => false,
        },
        _ => target() == slot,
    }
}

/// The tab that holds the selection (Favorites when it is a favourite).
fn initial_tab(page: Page, force: Option<&str>) -> usize {
    let slot = std::iter::once(Slot::Auto)
        .chain((0..RELEASES.len()).map(Slot::Release))
        .find(|s| selected(page, *s, force));
    let Some(slot) = slot else { return 1 };
    if m::is_favorite(slot.name()) {
        return 0;
    }
    match slot.release() {
        None => 1,
        Some(r) => Category::TABS.iter().position(|c| *c == r.category).unwrap_or(1),
    }
}

/// The rows of a protocol tab, or of a search.
pub fn protocol_rows(tab: usize, query: &str) -> Vec<Slot> {
    if !query.is_empty() {
        return (0..RELEASES.len())
            .filter(|&i| {
                let r = &RELEASES[i];
                r.name.to_lowercase().contains(query)
                    || r.name.split('-').any(|n| n.to_lowercase().contains(query))
                    || r.title.is_some_and(|t| t.to_lowercase().contains(query))
            })
            .map(Slot::Release)
            .collect();
    }
    let cat = Category::TABS.get(tab).copied().unwrap_or(Category::Modern);
    if cat == Category::Favorites {
        return std::iter::once(Slot::Auto)
            .chain((0..RELEASES.len()).map(Slot::Release))
            .filter(|s| m::is_favorite(s.name()))
            .collect();
    }
    let mut rows = Vec::new();
    if cat == Category::Modern {
        rows.push(Slot::Auto);
    }
    rows.extend((0..RELEASES.len()).filter(|&i| RELEASES[i].category == cat).map(Slot::Release));
    rows
}

pub fn setting_rows(query: &str) -> Vec<usize> {
    (0..SETTINGS.len())
        .filter(|&i| query.is_empty() || SETTINGS[i].to_lowercase().contains(query))
        .collect()
}

// ---------------------------------------------------------------- geometry

/// `TabNavigationBar` + ViaFabricPlus's widening: the first tab's x and every
/// tab's width, for labels as wide as `label_w`.
pub fn tab_bar(w: f32, n: usize, label_w: f32) -> (f32, f32) {
    let even = |v: f32| (v / 2.0).ceil() * 2.0;
    let i = w.min(400.0) - 28.0;
    let mut tw = even((i / n as f32).trunc());
    let mut x0 = even(((w - i) / 2.0).trunc());
    let wide = (((label_w + 8.0).min(((w - 28.0) / n as f32).trunc())) as i32 & !1) as f32;
    if wide > tw {
        tw = wide;
        x0 = even(((w - tw * n as f32) / 2.0).trunc());
    }
    (x0, tw)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct List {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub width: f32,
    pub item: f32,
    pub content: f32,
    pub bar_x: f32,
}

impl List {
    pub fn new(kind: ListKind, size: [f32; 2], rows: usize) -> Self {
        let [w, h] = size;
        let width = kind.row_width(w);
        let item = kind.item_height();
        let left = (w / 2.0).floor() - (width / 2.0).floor();
        Self {
            top: LIST_TOP,
            bottom: h - kind.bottom_margin(),
            left,
            width,
            item,
            content: rows as f32 * item + 4.0,
            bar_x: left + width + 8.0,
        }
    }
    /// Row `i`'s content rectangle (the row less its 2-pixel padding).
    pub fn row(&self, i: usize, scroll: f32) -> [f32; 4] {
        let y = self.top + 2.0 - scroll.trunc() + i as f32 * self.item;
        [self.left + 2.0, y + 2.0, self.width - 4.0, self.item - 4.0]
    }
    pub fn row_at(&self, at: [f32; 2], scroll: f32, n: usize) -> Option<usize> {
        if at[1] < self.top || at[1] >= self.bottom || at[0] < self.left || at[0] >= self.left + self.width {
            return None;
        }
        let i = ((at[1] - self.top - 2.0 + scroll.trunc()) / self.item).floor();
        (i >= 0.0 && (i as usize) < n).then_some(i as usize)
    }
}

/// The search box's rectangle.
pub fn search_rect(w: f32) -> [f32; 4] {
    let sw = 260.0_f32.min(w - 20.0);
    [((w - sw) / 2.0).trunc(), SEARCH_TOP, sw, 20.0]
}

pub const BACK: [f32; 4] = [5.0, 5.0, 60.0, 20.0];

/// The footer's buttons, centred in the bottom 30 pixels.
pub fn footer(size: [f32; 2], n: usize) -> Vec<[f32; 4]> {
    let [w, h] = size;
    let bw = BUTTON_W.min(((w - (n as f32 + 1.0) * 4.0) / n as f32).trunc());
    let total = bw * n as f32 + 4.0 * (n as f32 - 1.0);
    let x0 = ((w - total) / 2.0).trunc();
    let y = h - FOOTER_H + ((FOOTER_H - 20.0) / 2.0).trunc();
    (0..n).map(|i| [x0 + i as f32 * (bw + 4.0), y, bw, 20.0]).collect()
}

pub fn footer_labels(page: Page) -> Vec<(&'static str, bool)> {
    match page {
        // Server lists connect to BetaCraft and ClassiCube servers, which Horizon can't.
        Page::Protocols => vec![("Server lists", false), ("Settings", true), ("Report issues", true)],
        Page::Force => vec![("Reset", true)],
        _ => Vec::new(),
    }
}

/// Report issues' popup: panel, then its buttons and the cross.
pub struct Popup {
    pub panel: [f32; 4],
    pub buttons: Vec<[f32; 4]>,
    pub cross: [f32; 4],
}

pub fn popup(size: [f32; 2]) -> Popup {
    let [w, h] = size;
    let border: f32 = 6.0;
    let body_w: f32 = 200.0 + 2.0 * 3.0;
    let body_h: f32 = 4.0 * (20.0 + 3.0) + 3.0;
    let pw = body_w.min(w - 2.0 * border);
    let ph = (20.0 + body_h).min(h - 2.0 * border);
    let panel = [((w - pw) / 2.0).trunc(), ((h - ph) / 2.0).trunc(), pw, ph];
    let body = [panel[0], panel[1] + 20.0, pw, ph - 20.0];
    let buttons = (0..REPORT.len())
        .map(|i| [body[0] + ((body[2] - 200.0) / 2.0).trunc(), body[1] + 3.0 + i as f32 * 23.0, 200.0, 20.0])
        .collect();
    let cross = [panel[0] + pw - 14.0 - 3.0, panel[1] + 3.0, 14.0, 14.0];
    Popup { panel, buttons, cross }
}

pub fn inside(r: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= r[0] && at[0] < r[0] + r[2] && at[1] >= r[1] && at[1] < r[1] + r[3]
}

// ---------------------------------------------------------------- events

/// What an event asks the game to do: settings to store, a link to open.
#[derive(Default)]
pub struct Outcome {
    pub store: Vec<(&'static str, String)>,
    pub open: Option<String>,
    pub logs: bool,
}

fn query(ui: &Ui) -> String {
    ui.search.value.trim().to_lowercase()
}

fn list_of(ui: &Ui, page: Page, size: [f32; 2]) -> Option<(ListKind, List, usize)> {
    let kind = page.list()?;
    let q = query(ui);
    let n = match kind {
        ListKind::Protocols => protocol_rows(ui.tab[0], &q).len(),
        ListKind::Settings => setting_rows(&q).len(),
    };
    Some((kind, List::new(kind, size, n), n))
}

/// The scroll of the list shown (the search's while searching).
pub fn scroll_mut(ui: &mut Ui, kind: ListKind) -> &mut crate::ui::Scroll {
    if !ui.search.value.trim().is_empty() {
        return &mut ui.search_scroll;
    }
    let tab = ui.tab[kind as usize].min(7);
    &mut ui.scroll[kind as usize][tab]
}

/// A left click at `at` on a `size` screen (`tw` measures text).
pub fn click(size: [f32; 2], at: [f32; 2], atlas: &crate::hud::HudAtlas) -> Outcome {
    let mut out = Outcome::default();
    let Some(page) = page() else { return out };
    if page == Page::Report {
        let p = popup(size);
        if inside(p.cross, at) {
            back();
            return out;
        }
        for (i, r) in p.buttons.iter().enumerate() {
            if inside(*r, at) && REPORT[i].1 {
                match i {
                    0 => out.open = Some(format!("{REPO}/issues/new?labels=bug")),
                    1 => out.open = Some(format!("{REPO}/issues/new?labels=enhancement")),
                    _ => out.logs = true,
                }
            }
        }
        return out;
    }
    if inside(BACK, at) {
        back();
        return out;
    }
    let sr = search_rect(size[0]);
    with(|ui| {
        ui.search.focused = inside(sr, at);
        if ui.search.focused {
            ui.search.x = sr[0];
            ui.search.y = sr[1];
            ui.search.width = sr[2];
            ui.search.click(atlas, at[0], false, false);
        }
    });
    // Footer.
    let labels = footer_labels(page);
    for (i, r) in footer(size, labels.len()).iter().enumerate() {
        if inside(*r, at) && labels[i].1 {
            match (page, i) {
                (Page::Protocols, 1) => open(Page::Settings, None),
                (Page::Protocols, 2) => open(Page::Report, None),
                (Page::Force, 0) => {
                    if let Some(a) = with(|ui| ui.force.clone()) {
                        out.store.push((m::SERVERS_KEY, m::set_forced(&a, None)));
                    }
                }
                _ => {}
            }
            return out;
        }
    }
    let Some(kind) = page.list() else { return out };
    // Tabs.
    let tabs = kind.tabs();
    let label_w = tabs.iter().map(|t| atlas.font.width(t)).fold(0.0, f32::max);
    let (x0, tw) = tab_bar(size[0], tabs.len(), label_w);
    if at[1] >= CONTENT_TOP && at[1] < CONTENT_TOP + TAB_H && at[0] >= x0 {
        let i = ((at[0] - x0) / tw) as usize;
        if i < tabs.len() {
            with(|ui| {
                ui.tab[kind as usize] = i;
                ui.search.set_value("");
            });
        }
        return out;
    }
    // The list: its scrollbar, then a row.
    let (list, n, scroll_amount, pressed) = with(|ui| {
        let (_, list, n) = list_of(ui, page, size).unwrap();
        let s = scroll_mut(ui, kind);
        s.clamp(list.top, list.bottom, list.content);
        let pressed = s.press((at[0], at[1]), list.bar_x, list.top, list.bottom, list.content);
        (list, n, s.amount, pressed)
    });
    if pressed {
        return out;
    }
    let Some(i) = list.row_at(at, scroll_amount, n) else { return out };
    let row = list.row(i, scroll_amount);
    match kind {
        ListKind::Protocols => {
            let q = with(|ui| query(ui));
            let tab = with(|ui| ui.tab[0]);
            let slot = protocol_rows(tab, &q)[i];
            if at[0] >= row[0] + row[2] - STAR_AREA {
                out.store.push((m::FAVORITES_KEY, m::toggle_favorite(slot.name())));
                return out;
            }
            if !slot.joinable() {
                return out;
            }
            match page {
                Page::Force => {
                    if let Some(a) = with(|ui| ui.force.clone()) {
                        out.store.push((m::SERVERS_KEY, m::set_forced(&a, slot.version())));
                    }
                }
                _ => out.store.extend(m::set_target(slot.version())),
            }
        }
        ListKind::Settings => {
            let q = with(|ui| query(ui));
            if let Some(&s) = setting_rows(&q).get(i) {
                out.store.extend(m::setting_click(s));
            }
        }
    }
    out
}

pub fn wheel(size: [f32; 2], delta: f32) {
    let Some(page) = page() else { return };
    with(|ui| {
        let Some((kind, list, n)) = list_of(ui, page, size) else { return };
        // ViaFabricPlus's rate: whole rows once a list is long.
        let steps = (n / 5).min(2) as f32;
        let rate = (list.item / 2.0).max(steps * list.item);
        scroll_mut(ui, kind).wheel(delta, rate, list.top, list.bottom, list.content);
    })
}

pub fn drag(size: [f32; 2], y: f32) {
    let Some(page) = page() else { return };
    with(|ui| {
        let Some((kind, list, _)) = list_of(ui, page, size) else { return };
        scroll_mut(ui, kind).drag(y, list.top, list.bottom, list.content);
    })
}

pub fn release() {
    with(|ui| {
        for kind in [ListKind::Protocols, ListKind::Settings] {
            scroll_mut(ui, kind).release();
        }
    })
}

/// A key: Escape goes back; the search box takes the rest while focused.
/// True when it was the screen's.
pub fn key(name: &str, shift: bool, ctrl: bool) -> bool {
    if page().is_none() {
        return false;
    }
    if name == "key.keyboard.escape" {
        back();
        return true;
    }
    with(|ui| {
        if !ui.search.focused {
            return;
        }
        if let Some(code) = keycode(name) {
            ui.search.key(code, ctrl, shift);
        }
    });
    true
}

pub fn typed(c: char) -> bool {
    if page().is_none() {
        return false;
    }
    with(|ui| {
        if ui.search.focused && !c.is_control() {
            ui.search.insert_text(&c.to_string());
        }
    });
    true
}

fn keycode(name: &str) -> Option<winit::keyboard::KeyCode> {
    use winit::keyboard::KeyCode as K;
    Some(match name.strip_prefix("key.keyboard.")? {
        "backspace" => K::Backspace,
        "delete" => K::Delete,
        "left" => K::ArrowLeft,
        "right" => K::ArrowRight,
        "home" => K::Home,
        "end" => K::End,
        "a" => K::KeyA,
        "c" => K::KeyC,
        "v" => K::KeyV,
        "x" => K::KeyX,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_sit_like_viafabricplus() {
        assert_eq!(Orientation::RightTop.rect([427.0, 240.0]), Some([324.0, 5.0, 98.0, 20.0]));
        assert_eq!(Orientation::LeftBottom.rect([427.0, 240.0]), Some([5.0, 215.0, 98.0, 20.0]));
        assert_eq!(Orientation::None.rect([427.0, 240.0]), None);
    }

    #[test]
    fn tabs_widen_to_their_labels() {
        // 427 wide, 7 tabs: vanilla's 400-28 split gives 54; "April Fools" (55) + 8 fits in (427-28)/7=57.
        let (x0, tw) = tab_bar(427.0, 7, 55.0);
        assert_eq!(tw, 56.0);
        assert_eq!(x0 + tw * 7.0 <= 427.0, true);
    }

    #[test]
    fn modern_starts_with_auto_detect_and_26_3() {
        let rows = protocol_rows(1, "");
        assert_eq!(rows[0], Slot::Auto);
        assert_eq!(rows[1].name(), "26.3");
        assert!(rows[1].joinable() && !protocol_rows(2, "")[0].joinable());
        assert!(protocol_rows(0, "craftmine").iter().all(|s| s.name() == "25w14craftmine"));
    }

    #[test]
    fn list_rows_follow_abstract_selection_list() {
        let l = List::new(ListKind::Protocols, [427.0, 240.0], 10);
        assert_eq!(l.left, 213.0 - 130.0);
        assert_eq!(l.row(0, 0.0), [85.0, 88.0, 256.0, 28.0]);
        assert_eq!(l.row_at([100.0, 90.0], 0.0, 10), Some(0));
        assert_eq!(l.row_at([100.0, 130.0], 0.0, 10), Some(1));
    }
}
