// Multiplayer ekranının üstünde açılan sürüm seçim paneli. Horizon başlık
// ekranlarında modun kendi ekranını açtırmadığı için panel `screen_render_post`
// ile çizilir, olaylar `screen_*_pre` hook'larında yutulur. Burada yalnız saf
// yerleşim ve olay mantığı var. `horizon_client::net` içine
// `older_servers_panel` olarak eklenir.
use std::sync::Mutex;

use horizon_protocol::adapter::ServerVersion;

use super::older_servers_skin::{self as skin, ButtonState, Draw};

/// Panelin durumu: açık mı, listenin ilk görünen satırı.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Panel {
    pub open: bool,
    pub scroll: usize,
}

pub static PANEL: Mutex<Panel> = Mutex::new(Panel { open: false, scroll: 0 });

pub const ROW_HEIGHT: f32 = 20.0;
const MARGIN: f32 = 8.0;
const HEADER: f32 = 22.0;
const FOOTER: f32 = 28.0;
const PAD: f32 = 4.0;
const SCROLLBAR: f32 = 6.0;

/// Tıklanabilir yerler, çizildikleri sırayla; `Panel` en sonda, panelin kendisi.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Row(usize, [f32; 4]),
    AutoDetect([f32; 4]),
    Done([f32; 4]),
    Panel([f32; 4]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelEvent {
    Select(usize),
    ToggleAuto,
    Close,
    /// Panelin dışına tıklandı: kapanır, tık alttaki ekrana gitmez.
    Outside,
    None,
}

/// Panelin yerleri, `n` seçenekle `size` boyutlu ekranda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub panel: [f32; 4],
    pub list: [f32; 4],
    pub visible: usize,
    pub auto: [f32; 4],
    pub done: [f32; 4],
}

/// Ekrana sığan en çok satır.
pub fn max_rows(height: f32) -> usize {
    (((height - 2.0 * MARGIN - HEADER - FOOTER - PAD) / ROW_HEIGHT).floor() as isize).max(1) as usize
}

pub fn geometry(size: [f32; 2], n: usize) -> Geometry {
    let visible = n.min(max_rows(size[1])).max(1);
    let scrolls = n > visible;
    let width = (size[0] * 0.6).floor().max(160.0).min(size[0] - 2.0 * MARGIN);
    let height = HEADER + visible as f32 * ROW_HEIGHT + PAD + FOOTER;
    let x = ((size[0] - width) / 2.0).floor();
    let y = ((size[1] - height) / 2.0).floor().max(0.0);
    let list_w = width - 2.0 * PAD - if scrolls { SCROLLBAR + 2.0 } else { 0.0 };
    let list = [x + PAD, y + HEADER, list_w, visible as f32 * ROW_HEIGHT];
    let by = y + HEADER + visible as f32 * ROW_HEIGHT + PAD + 4.0;
    let bw = ((width - 3.0 * PAD) / 2.0).floor();
    Geometry {
        panel: [x, y, width, height],
        list,
        visible,
        auto: [x + PAD, by, bw, 20.0],
        done: [x + width - PAD - bw, by, bw, 20.0],
    }
}

/// Listenin olabilecek en aşağı kaydırması.
pub fn clamp_scroll(scroll: usize, n: usize, visible: usize) -> usize {
    scroll.min(n.saturating_sub(visible))
}

/// Seçili satırı gösteren kaydırma (gerekmedikçe yerinde kalır).
pub fn scroll_to_show(selected: usize, scroll: usize, n: usize, visible: usize) -> usize {
    let scroll = if selected < scroll {
        selected
    } else if selected >= scroll + visible {
        selected + 1 - visible
    } else {
        scroll
    };
    clamp_scroll(scroll, n, visible)
}

/// Panelin çizimi ve tıklanabilir yerleri.
#[allow(clippy::too_many_arguments)]
pub fn layout(
    size: [f32; 2],
    items: &[Option<ServerVersion>],
    selected: usize,
    scroll: usize,
    mouse: [f32; 2],
    auto_detect: bool,
    tw: &dyn Fn(&str) -> f32,
) -> (Vec<Draw>, Vec<Hit>) {
    let g = geometry(size, items.len());
    let scroll = clamp_scroll(scroll, items.len(), g.visible);
    let inside = super::older_servers_mod::inside;
    let mut draws = vec![
        // Arkadaki ekran karartılır, panel vanilla menü kutusu gibi koyu.
        Draw::Fill([0.0, 0.0, size[0], size[1]], 0xA000_0000),
        Draw::Fill(g.panel, 0xF010_1010),
    ];
    let [px, py, pw, ph] = g.panel;
    draws.extend([
        Draw::Fill([px, py, pw, 1.0], 0xFF55_5555),
        Draw::Fill([px, py + ph - 1.0, pw, 1.0], 0xFF55_5555),
        Draw::Fill([px, py, 1.0, ph], 0xFF55_5555),
        Draw::Fill([px + pw - 1.0, py, 1.0, ph], 0xFF55_5555),
    ]);
    let title = "Sürüm seç";
    draws.push(Draw::Text(
        title.into(),
        [px + (pw - tw(title)) / 2.0, py + 7.0],
        0xFF_FFFF,
        true,
    ));
    let mut hits = Vec::new();
    for (row, i) in (scroll..items.len()).take(g.visible).enumerate() {
        let rect = [g.list[0], g.list[1] + row as f32 * ROW_HEIGHT, g.list[2], ROW_HEIGHT];
        let label = super::older_servers_mod::choice_label(items[i], auto_detect);
        draws.extend(skin::list_row(rect, i == selected, inside(rect, mouse), &label, tw(&label)));
        hits.push(Hit::Row(i, rect));
    }
    let track = [g.list[0] + g.list[2] + 2.0, g.list[1], SCROLLBAR, g.list[3]];
    draws.extend(skin::scrollbar(
        track,
        scroll as f32,
        g.visible as f32,
        items.len() as f32,
    ));
    let auto = format!("Otomatik algılama: {}", if auto_detect { "AÇIK" } else { "KAPALI" });
    for (rect, label) in [(g.auto, auto.as_str()), (g.done, "Tamam")] {
        let state = if inside(rect, mouse) { ButtonState::Hover } else { ButtonState::Normal };
        // Etiket butona sığmazsa kısaltılır.
        let label = fit(label, rect[2] - 8.0, tw);
        draws.extend(skin::button(rect, state, &label, tw(&label)));
    }
    hits.push(Hit::AutoDetect(g.auto));
    hits.push(Hit::Done(g.done));
    hits.push(Hit::Panel(g.panel));
    (draws, hits)
}

/// `label` `width`'e sığmazsa sonu `…` ile kesilir.
fn fit(label: &str, width: f32, tw: &dyn Fn(&str) -> f32) -> String {
    if tw(label) <= width {
        return label.to_owned();
    }
    let mut s: String = label.to_owned();
    while !s.is_empty() && tw(&format!("{s}…")) > width {
        s.pop();
    }
    format!("{s}…")
}

/// Tıklanan yer.
pub fn click(hits: &[Hit], at: [f32; 2]) -> PanelEvent {
    let inside = super::older_servers_mod::inside;
    for hit in hits {
        match *hit {
            Hit::Row(i, r) if inside(r, at) => return PanelEvent::Select(i),
            Hit::AutoDetect(r) if inside(r, at) => return PanelEvent::ToggleAuto,
            Hit::Done(r) if inside(r, at) => return PanelEvent::Close,
            Hit::Panel(r) if inside(r, at) => return PanelEvent::None,
            _ => {}
        }
    }
    PanelEvent::Outside
}

/// Tekerlek: yukarı (pozitif) listeyi yukarı kaydırır.
pub fn wheel(scroll: usize, delta: f64, n: usize, visible: usize) -> usize {
    let scroll = if delta > 0.0 {
        scroll.saturating_sub(1)
    } else if delta < 0.0 {
        scroll + 1
    } else {
        scroll
    };
    clamp_scroll(scroll, n, visible)
}

/// Panel açıkken bir tuşun anlamı.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Close,
    Up,
    Down,
    Select,
    Ignore,
}

pub fn key(name: &str) -> KeyAction {
    match name {
        "key.keyboard.escape" => KeyAction::Close,
        "key.keyboard.up" => KeyAction::Up,
        "key.keyboard.down" => KeyAction::Down,
        "key.keyboard.enter" | "key.keyboard.keypad.enter" | "key.keyboard.space" => KeyAction::Select,
        _ => KeyAction::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tw(s: &str) -> f32 {
        s.chars().count() as f32 * 6.0
    }

    fn rows(hits: &[Hit]) -> Vec<(usize, [f32; 4])> {
        hits.iter()
            .filter_map(|h| match h {
                Hit::Row(i, r) => Some((*i, *r)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn five_versions_are_five_centred_rows_and_two_buttons() {
        let items: Vec<_> = ServerVersion::ALL.into_iter().map(Some).collect();
        let (_, hits) = layout([320.0, 240.0], &items, 0, 0, [0.0, 0.0], true, &tw);
        let rows = rows(&hits);
        assert_eq!(rows.len(), 5);
        assert!(rows.iter().all(|(_, r)| r[3] == ROW_HEIGHT));
        // Satırlar alt alta, panelin ortasında.
        let g = geometry([320.0, 240.0], 5);
        for (k, (i, r)) in rows.iter().enumerate() {
            assert_eq!(*i, k);
            assert_eq!(r[1], g.list[1] + k as f32 * ROW_HEIGHT);
            let centre = r[0] + r[2] / 2.0;
            assert!((centre - 160.0).abs() <= 1.0, "{centre}");
        }
        let buttons = hits
            .iter()
            .filter(|h| matches!(h, Hit::AutoDetect(_) | Hit::Done(_)))
            .count();
        assert_eq!(buttons, 2);
    }

    #[test]
    fn clicks_select_toggle_close_or_fall_outside() {
        let items: Vec<_> = ServerVersion::ALL.into_iter().map(Some).collect();
        let (_, hits) = layout([320.0, 240.0], &items, 0, 0, [0.0, 0.0], true, &tw);
        let second = rows(&hits)[1].1;
        assert_eq!(click(&hits, [second[0] + 5.0, second[1] + 5.0]), PanelEvent::Select(1));
        let g = geometry([320.0, 240.0], 5);
        assert_eq!(click(&hits, [g.auto[0] + 1.0, g.auto[1] + 1.0]), PanelEvent::ToggleAuto);
        assert_eq!(click(&hits, [g.done[0] + 1.0, g.done[1] + 1.0]), PanelEvent::Close);
        // Panelin içi ama hiçbir şeyin üstü değil: başlık.
        assert_eq!(click(&hits, [g.panel[0] + 2.0, g.panel[1] + 2.0]), PanelEvent::None);
        assert_eq!(click(&hits, [1.0, 1.0]), PanelEvent::Outside);
    }

    #[test]
    fn twenty_versions_scroll_and_show_the_selected_one() {
        let items: Vec<Option<ServerVersion>> = vec![Some(ServerVersion::Current); 20];
        let g = geometry([320.0, 240.0], 20);
        assert_eq!(g.visible, 8);
        assert_eq!(clamp_scroll(15, 20, g.visible), 12);
        assert_eq!(scroll_to_show(18, 0, 20, g.visible), 11);
        assert_eq!(scroll_to_show(3, 11, 20, g.visible), 3);
        assert_eq!(scroll_to_show(5, 0, 20, g.visible), 0);
        let (draws, hits) = layout([320.0, 240.0], &items, 18, 15, [0.0, 0.0], true, &tw);
        let shown: Vec<usize> = rows(&hits).iter().map(|(i, _)| *i).collect();
        assert_eq!(shown, (12..20).collect::<Vec<_>>());
        assert!(draws.iter().any(|d| matches!(d, Draw::Sprite { name, .. } if name == skin::SCROLLER)));
        assert_eq!(wheel(12, -1.0, 20, 8), 12);
        assert_eq!(wheel(12, 1.0, 20, 8), 11);
        assert_eq!(wheel(0, 1.0, 20, 8), 0);
    }

    #[test]
    fn the_panel_fits_a_small_and_a_large_screen() {
        for size in [[320.0, 240.0], [480.0, 270.0], [960.0, 540.0], [213.0, 120.0]] {
            let g = geometry(size, 6);
            assert!(g.panel[0] >= 0.0 && g.panel[0] + g.panel[2] <= size[0], "{size:?}");
            assert!(g.panel[1] >= 0.0, "{size:?}");
            assert!(g.done[0] + g.done[2] <= g.panel[0] + g.panel[2]);
        }
    }

    #[test]
    fn keys_close_move_or_select() {
        assert_eq!(key("key.keyboard.escape"), KeyAction::Close);
        assert_eq!(key("key.keyboard.down"), KeyAction::Down);
        assert_eq!(key("key.keyboard.enter"), KeyAction::Select);
        assert_eq!(key("key.keyboard.a"), KeyAction::Ignore);
    }

    #[test]
    fn a_long_label_is_cut_to_fit() {
        assert_eq!(fit("abcdef", 100.0, &tw), "abcdef");
        let cut = fit("abcdefghij", 30.0, &tw);
        assert!(tw(&cut) <= 30.0 && cut.ends_with('…'), "{cut}");
    }
}
