// Vanilla görünümlü çizim parçaları: buton, liste satırı, kaydırma çubuğu. Hepsi
// oyunun atlasındaki vanilla sprite'larından adla çizilir (PNG kopyalanmaz);
// burada yalnız ne çizileceği (`Draw`) üretilir, çizimi ekran hook'u yapar.
// `horizon_client::net` içine `older_servers_skin` olarak eklenir.

/// Çizilecek tek parça. `Fill` ARGB, `Text` RGB (gölgeli ya da değil), `Sprite`
/// sprite'ın kendi piksellerindeki `src` parçası GUI piksellerindeki `dst`'ye.
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    Fill([f32; 4], u32),
    Text(String, [f32; 2], u32, bool),
    Sprite { name: String, src: [f32; 4], dst: [f32; 4] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Disabled,
    /// Seçili (açık panelin butonu gibi): vurgulu kenar.
    Selected,
}

/// `gui/sprites/widget/button*.png.mcmeta`: 200×20, kenar 3 (nine_slice).
pub const BUTTON: &str = "gui/sprites/widget/button";
pub const BUTTON_HIGHLIGHTED: &str = "gui/sprites/widget/button_highlighted";
pub const BUTTON_DISABLED: &str = "gui/sprites/widget/button_disabled";
const BUTTON_SIZE: [f32; 2] = [200.0, 20.0];
const BUTTON_BORDER: f32 = 3.0;
/// `gui/sprites/widget/scroller*.png.mcmeta`: 6×32, kenar 1 (nine_slice).
pub const SCROLLER: &str = "gui/sprites/widget/scroller";
pub const SCROLLER_BACKGROUND: &str = "gui/sprites/widget/scroller_background";
const SCROLLER_SIZE: [f32; 2] = [6.0, 32.0];
const SCROLLER_BORDER: f32 = 1.0;

const WHITE: u32 = 0xFF_FFFF;
const GRAY: u32 = 0xA0_A0A0;

/// Bir eksende nine-slice parçaları: (kaynak başı, kaynak boyu, hedef başı, hedef boyu).
/// Köşeler kendi boyunda, orta kısım kaynaktan döşenir (vanilla'nın `tile`'ı gibi:
/// son parça kesilir).
fn spans(size: f32, border: f32, at: f32, len: f32) -> Vec<(f32, f32, f32, f32)> {
    let b = border.min((len / 2.0).floor()).max(0.0);
    let mut out = Vec::new();
    if b > 0.0 {
        out.push((0.0, b, at, b));
    }
    let middle = len - 2.0 * b;
    let src_middle = size - 2.0 * border;
    if middle > 0.0 && src_middle > 0.0 {
        let mut done = 0.0;
        while done < middle {
            let piece = src_middle.min(middle - done);
            out.push((border, piece, at + b + done, piece));
            done += piece;
        }
    }
    if b > 0.0 {
        out.push((size - b, b, at + len - b, b));
    }
    out
}

/// `blitNineSlicedSprite`: `name` sprite'ı (`size` piksel, `border` kenar) `dst`'ye.
pub fn nine_slice(name: &str, size: [f32; 2], border: f32, dst: [f32; 4]) -> Vec<Draw> {
    let mut out = Vec::new();
    for (sy, sh, dy, dh) in spans(size[1], border, dst[1], dst[3]) {
        for (sx, sw, dx, dw) in spans(size[0], border, dst[0], dst[2]) {
            out.push(Draw::Sprite {
                name: name.to_owned(),
                src: [sx, sy, sw, sh],
                dst: [dx, dy, dw, dh],
            });
        }
    }
    out
}

fn outline(rect: [f32; 4], argb: u32) -> [Draw; 4] {
    let [x, y, w, h] = rect;
    [
        Draw::Fill([x, y, w, 1.0], argb),
        Draw::Fill([x, y + h - 1.0, w, 1.0], argb),
        Draw::Fill([x, y, 1.0, h], argb),
        Draw::Fill([x + w - 1.0, y, 1.0, h], argb),
    ]
}

/// Ortalanmış etiket (`AbstractWidget.renderScrollingString`'in sığan hali).
fn centered(rect: [f32; 4], label: &str, text_w: f32, color: u32) -> Draw {
    let x = rect[0] + (rect[2] - text_w) / 2.0;
    let y = rect[1] + ((rect[3] - 8.0) / 2.0).floor();
    Draw::Text(label.to_owned(), [x, y], color, true)
}

/// Vanilla buton.
pub fn button(rect: [f32; 4], state: ButtonState, label: &str, text_w: f32) -> Vec<Draw> {
    let sprite = match state {
        ButtonState::Normal => BUTTON,
        ButtonState::Hover | ButtonState::Selected => BUTTON_HIGHLIGHTED,
        ButtonState::Disabled => BUTTON_DISABLED,
    };
    let mut out = nine_slice(sprite, BUTTON_SIZE, BUTTON_BORDER, rect);
    if state == ButtonState::Selected {
        out.extend(outline(rect, 0xFFFF_FFFF));
    }
    let color = if state == ButtonState::Disabled { GRAY } else { WHITE };
    out.push(centered(rect, label, text_w, color));
    out
}

/// Liste satırı (`AbstractSelectionList`): seçiliyse beyaz kenar içinde siyah,
/// üzerindeyse hafif aydınlık.
pub fn list_row(rect: [f32; 4], selected: bool, hover: bool, label: &str, text_w: f32) -> Vec<Draw> {
    let [x, y, w, h] = rect;
    let mut out = Vec::new();
    if selected {
        out.extend(outline(rect, 0xFFFF_FFFF));
        out.push(Draw::Fill([x + 1.0, y + 1.0, w - 2.0, h - 2.0], 0xFF00_0000));
    } else if hover {
        out.push(Draw::Fill(rect, 0x30FF_FFFF));
    }
    out.push(centered(rect, label, text_w, WHITE));
    out
}

/// Tutamacın y'si ve boyu: `offset` (satır) / `total - visible` kadar aşağıda,
/// `track.h * visible / total` uzunluğunda.
pub fn thumb(track: [f32; 4], offset: f32, visible: f32, total: f32) -> Option<(f32, f32)> {
    if total <= visible || total <= 0.0 {
        return None;
    }
    let h = (track[3] * visible / total).max(8.0).min(track[3]);
    let max = total - visible;
    let y = track[1] + (track[3] - h) * (offset.clamp(0.0, max) / max);
    Some((y, h))
}

/// Kaydırma çubuğu: zemin ve tutamaç; hepsi sığıyorsa hiçbir şey.
pub fn scrollbar(track: [f32; 4], offset: f32, visible: f32, total: f32) -> Vec<Draw> {
    let Some((y, h)) = thumb(track, offset, visible, total) else {
        return Vec::new();
    };
    let mut out = nine_slice(SCROLLER_BACKGROUND, SCROLLER_SIZE, SCROLLER_BORDER, track);
    out.extend(nine_slice(SCROLLER, SCROLLER_SIZE, SCROLLER_BORDER, [track[0], y, track[2], h]));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprites(d: &[Draw]) -> Vec<&str> {
        d.iter()
            .filter_map(|d| match d {
                Draw::Sprite { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_hovered_button_is_the_highlighted_sprite_with_a_centred_label() {
        let rect = [10.0, 20.0, 100.0, 20.0];
        let d = button(rect, ButtonState::Hover, "Sürüm", 30.0);
        assert!(sprites(&d).iter().all(|s| *s == BUTTON_HIGHLIGHTED));
        assert!(!sprites(&d).is_empty());
        let text = d.iter().find_map(|d| match d {
            Draw::Text(t, at, color, _) => Some((t.clone(), *at, *color)),
            _ => None,
        });
        assert_eq!(text, Some(("Sürüm".into(), [10.0 + (100.0 - 30.0) / 2.0, 26.0], WHITE)));
        assert!(sprites(&button(rect, ButtonState::Normal, "a", 5.0)).iter().all(|s| *s == BUTTON));
        assert!(sprites(&button(rect, ButtonState::Disabled, "a", 5.0)).iter().all(|s| *s == BUTTON_DISABLED));
    }

    #[test]
    fn a_button_is_nine_sliced_and_covers_its_rectangle() {
        let rect = [0.0, 0.0, 98.0, 20.0];
        let d = nine_slice(BUTTON, BUTTON_SIZE, BUTTON_BORDER, rect);
        // 3 sütun × 3 satır; köşeler 3 piksel, orta kaynağın baştan kesilmiş hali.
        assert_eq!(d.len(), 9);
        let area: f32 = d
            .iter()
            .map(|d| match d {
                Draw::Sprite { dst, .. } => dst[2] * dst[3],
                _ => 0.0,
            })
            .sum();
        assert_eq!(area, 98.0 * 20.0);
        assert!(d.contains(&Draw::Sprite {
            name: BUTTON.into(),
            src: [197.0, 17.0, 3.0, 3.0],
            dst: [95.0, 17.0, 3.0, 3.0],
        }));
        // 200'den geniş buton: orta kısım iki parça döşenir.
        assert_eq!(nine_slice(BUTTON, BUTTON_SIZE, BUTTON_BORDER, [0.0, 0.0, 300.0, 20.0]).len(), 12);
    }

    #[test]
    fn a_selected_row_has_a_white_outline_round_black() {
        let rect = [0.0, 0.0, 100.0, 20.0];
        let d = list_row(rect, true, false, "1.21.11", 30.0);
        assert!(d.contains(&Draw::Fill([0.0, 0.0, 100.0, 1.0], 0xFFFF_FFFF)));
        assert!(d.contains(&Draw::Fill([0.0, 19.0, 100.0, 1.0], 0xFFFF_FFFF)));
        assert!(d.contains(&Draw::Fill([1.0, 1.0, 98.0, 18.0], 0xFF00_0000)));
        let plain = list_row(rect, false, false, "1.21.11", 30.0);
        assert!(plain.iter().all(|d| matches!(d, Draw::Text(..))));
    }

    #[test]
    fn the_thumb_follows_the_offset() {
        let track = [0.0, 10.0, 6.0, 160.0];
        let (y, h) = thumb(track, 10.0, 5.0, 20.0).unwrap();
        assert_eq!(h, 160.0 * 5.0 / 20.0);
        assert_eq!(y, 10.0 + (160.0 - h) * 10.0 / 15.0);
        assert_eq!(thumb(track, 0.0, 5.0, 5.0), None);
        assert!(scrollbar(track, 0.0, 5.0, 5.0).is_empty());
        let d = scrollbar(track, 10.0, 5.0, 20.0);
        assert!(sprites(&d).contains(&SCROLLER_BACKGROUND));
        assert!(sprites(&d).contains(&SCROLLER));
    }
}
