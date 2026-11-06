// ViaFabricPlus's screens drawn over the title's Multiplayer screens with the
// game's own widgets (menu background, tabs, list, separators, scrollbar,
// buttons, edit box, tooltip): see `vfp.rs` for the state and the geometry.
// A module of `game::native_hooks`, for `Game`'s atlas and title. Pinned: it
// reads `Game::gui` and `Game::title`.
use horizon_client::hud::{HudBatch, rgba};
use horizon_client::multiplayer::older_servers_rows as rows;
use horizon_client::net::{older_servers_mod as m, older_servers_vfp as v};
use horizon_client::text::Text;
use horizon_client::ui;

fn col(rgb: u32) -> u32 {
    rgba((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 0xFF)
}

fn multiplayer<'a>(game: &'a crate::game::Game<'_>) -> Option<&'a horizon_client::multiplayer::Screen> {
    match game.title.as_ref().map(|t| &t.mode) {
        Some(horizon_client::title::Mode::Multiplayer(s)) => Some(s),
        _ => None,
    }
}

/// The screens the mod adds to: the server list, Add/Edit Server, Direct Connect.
pub(super) fn ours(id: &str) -> bool {
    matches!(id, "multiplayer" | "edit_server" | "add_server" | "direct_connect")
}

/// The 98x20 button the screen shows, and its label.
pub(super) fn screen_button(
    game: &crate::game::Game<'_>,
    id: &str,
    size: [f32; 2],
) -> Option<([f32; 4], String)> {
    let s = m::settings();
    match id {
        "multiplayer" => s.mp_button.rect(size).map(|r| (r, v::TITLE.to_owned())),
        "direct_connect" => s.direct_button.rect(size).map(|r| (r, v::TITLE.to_owned())),
        "edit_server" | "add_server" => {
            let address = rows::edit_address(multiplayer(game)?)?;
            let label = m::forced(address).map_or("Set version".to_owned(), |f| v::name_of(f).to_owned());
            s.add_button.rect(size).map(|r| (r, label))
        }
        _ => None,
    }
}

/// The address being added or edited (Force Version's server).
pub(super) fn edit_address(game: &crate::game::Game<'_>) -> Option<String> {
    rows::edit_address(multiplayer(game)?).map(str::to_owned)
}

pub(super) fn draw(
    game: &crate::game::Game<'_>,
    batch: &mut HudBatch,
    id: &str,
    size: [f32; 2],
    mouse: [f32; 2],
) {
    let atlas = &game.gui.atlas;
    let Some(page) = v::page() else {
        if let Some((rect, label)) = screen_button(game, id, size) {
            let hover = v::inside(rect, mouse);
            ui::button(batch, atlas, rect[0], rect[1], rect[2], rect[3], &Text::literal(label), true, hover);
        }
        if id == "multiplayer" && m::settings().show_advertised {
            ping_tooltip(game, batch, size, mouse);
        }
        return;
    };
    // The Multiplayer screen is covered: what it drew goes, the title's
    // panorama (drawn apart from this batch) stays behind the menu background.
    batch.vertices.clear();
    batch.blends.clear();
    batch.blur_at = 0;
    if page == v::Page::Report {
        // A popup over the screen it came from, dimmed.
        screen(game, batch, v::Page::Protocols, size, [-1.0, -1.0]);
        popup(game, batch, size, mouse);
    } else {
        screen(game, batch, page, size, mouse);
    }
}

fn screen(game: &crate::game::Game<'_>, batch: &mut HudBatch, page: v::Page, size: [f32; 2], mouse: [f32; 2]) {
    let atlas = &game.gui.atlas;
    let [w, h] = size;
    ui::menu_background(batch, atlas, 0.0, 0.0, w, h, false);
    let kind = if page == v::Page::Settings { v::ListKind::Settings } else { v::ListKind::Protocols };
    let (query, tab, force) = v::with(|ui| (ui.search.value.trim().to_lowercase(), ui.tab[kind as usize], ui.force.clone()));
    // The list.
    match kind {
        v::ListKind::Protocols => {
            let slots = v::protocol_rows(tab, &query);
            let list = v::List::new(kind, size, slots.len());
            let scroll = v::with(|ui| {
                let s = v::scroll_mut(ui, kind);
                s.clamp(list.top, list.bottom, list.content);
                s.amount
            });
            ui::list_background(batch, atlas, list.top, list.bottom, false);
            let first = batch.vertices.len();
            for (i, slot) in slots.iter().enumerate() {
                let r = list.row(i, scroll);
                if r[1] + r[3] < list.top || r[1] > list.bottom {
                    continue;
                }
                protocol_slot(game, batch, page, *slot, force.as_deref(), r);
            }
            batch.clip_since(first, 0.0, list.top, w, list.bottom);
            ui::separators(batch, atlas, list.top, list.bottom, false);
            ui::scrollbar(batch, atlas, list.bar_x, list.top, list.bottom, list.content, scroll);
        }
        v::ListKind::Settings => {
            let items = v::setting_rows(&query);
            let list = v::List::new(kind, size, items.len());
            let scroll = v::with(|ui| {
                let s = v::scroll_mut(ui, kind);
                s.clamp(list.top, list.bottom, list.content);
                s.amount
            });
            ui::list_background(batch, atlas, list.top, list.bottom, false);
            let first = batch.vertices.len();
            for (i, item) in items.iter().enumerate() {
                let r = list.row(i, scroll);
                if r[1] + r[3] < list.top || r[1] > list.bottom {
                    continue;
                }
                setting_row(game, batch, *item, r);
            }
            batch.clip_since(first, 0.0, list.top, w, list.bottom);
            ui::separators(batch, atlas, list.top, list.bottom, false);
            ui::scrollbar(batch, atlas, list.bar_x, list.top, list.bottom, list.content, scroll);
        }
    }
    tabs(game, batch, kind, tab, query.is_empty(), size, mouse);
    // Search box, with its grey italic hint while empty and unfocused.
    let sr = v::search_rect(w);
    let (empty, focused) = v::with(|ui| {
        ui.search.x = sr[0];
        ui.search.y = sr[1];
        ui.search.width = sr[2];
        ui.search.height = sr[3];
        ui.search.draw(batch, atlas, None);
        (ui.search.value.is_empty(), ui.search.focused)
    });
    if empty && !focused {
        let hint = Text::literal(horizon_client::lang::get_or("gui.selectWorld.search", "Search..."))
            .italic()
            .with_color(0xAAAAAA);
        batch.text_styled(atlas, &hint, sr[0] + 4.0, sr[1] + 6.0, u32::MAX, true);
    }
    // Back, the footer, the big title.
    let back = horizon_client::lang::get_or("gui.back", "Back");
    ui::button(batch, atlas, v::BACK[0], v::BACK[1], v::BACK[2], v::BACK[3], &Text::literal(back), true, v::inside(v::BACK, mouse));
    let labels = v::footer_labels(page);
    for ((label, active), r) in labels.iter().zip(v::footer(size, labels.len())) {
        ui::button(batch, atlas, r[0], r[1], r[2], r[3], &Text::literal(*label), *active, v::inside(r, mouse));
    }
    let tw = atlas.font.width(v::TITLE);
    let x = 2.0 * ((w / 4.0).trunc() - (tw / 2.0).trunc());
    ui::text_scaled(batch, atlas, &Text::literal(v::TITLE), x, 12.0, 2.0, col(v::ACCENT));
}

fn tabs(
    game: &crate::game::Game<'_>,
    batch: &mut HudBatch,
    kind: v::ListKind,
    open: usize,
    tab_shown: bool,
    size: [f32; 2],
    mouse: [f32; 2],
) {
    let atlas = &game.gui.atlas;
    let labels = kind.tabs();
    let label_w = labels.iter().map(|l| atlas.font.width(l)).fold(0.0, f32::max);
    let (x0, tw) = v::tab_bar(size[0], labels.len(), label_w);
    let y = v::CONTENT_TOP;
    let right = x0 + tw * labels.len() as f32;
    for (x, width) in [(0.0, x0), (right, size[0] - right)] {
        ui::tiled_sized(batch, atlas, "gui/header_separator", x, y + v::TAB_H - 2.0, width, 2.0, [32.0, 2.0], u32::MAX);
    }
    for (i, label) in labels.iter().enumerate() {
        let x = x0 + tw * i as f32;
        let selected = tab_shown && i == open;
        let hover = v::inside([x, y, tw, v::TAB_H], mouse);
        let sprite = match (selected, hover) {
            (true, true) => "gui/sprites/widget/tab_selected_highlighted",
            (true, false) => "gui/sprites/widget/tab_selected",
            (false, true) => "gui/sprites/widget/tab_highlighted",
            (false, false) => "gui/sprites/widget/tab",
        };
        ui::nine_slice(batch, atlas, sprite, x, y, tw, v::TAB_H, u32::MAX);
        let text = Text::literal(*label);
        if selected {
            ui::menu_background(batch, atlas, x + 2.0, y + 2.0, tw - 4.0, v::TAB_H - 2.0, false);
            let uw = atlas.font.width(label).floor().min(tw - 4.0);
            batch.fill(atlas, x + ((tw - uw) / 2.0).floor(), y + v::TAB_H - 2.0, uw, 1.0, u32::MAX);
        }
        ui::menu_tab_label(batch, atlas, &text, (x, y, tw, v::TAB_H), selected, true, horizon_client::hud_overlays::now_ms());
    }
}

fn text(batch: &mut HudBatch, atlas: &horizon_client::hud::HudAtlas, s: &str, x: f32, y: f32, rgb: u32) {
    batch.text_styled(atlas, &Text::literal(s), x, y, col(rgb), true);
}

fn protocol_slot(
    game: &crate::game::Game<'_>,
    batch: &mut HudBatch,
    page: v::Page,
    slot: v::Slot,
    force: Option<&str>,
    r: [f32; 4],
) {
    let atlas = &game.gui.atlas;
    let [cx, cy, cw, ch] = r;
    batch.fill(atlas, cx, cy, cw, ch, rgba(0, 0, 0, 0x80));
    let joinable = slot.joinable();
    let selected = v::selected(page, slot, force);
    if selected && joinable {
        batch.fill(atlas, cx, cy, cw, ch, rgba(0x58, 0xA6, 0xFF, 0x40));
    }
    let icon_at = [cx + v::SLOT_MARGIN, cy + ((ch - v::ICON) / 2.0).trunc()];
    if !game.native_item(batch, slot.icon(), icon_at, v::ICON) {
        game.native_item(batch, "minecraft:barrier", icon_at, v::ICON);
    }
    let name_color = if !joinable { v::GRAY } else if selected { v::ACCENT } else { 0xFFFFFF };
    let centered_y = cy + (ch / 2.0).trunc() - 4.0;
    let star_w = atlas.font.width(v::STAR);
    let star_color = if m::is_favorite(slot.name()) { v::FAVORITE } else { v::NO_FAVORITE };
    text(batch, atlas, v::STAR, cx + cw - v::STAR_AREA + ((v::STAR_AREA - star_w) / 2.0).trunc(), centered_y, star_color);
    let x = cx + v::TEXT_OFFSET;
    let Some(release) = slot.release() else {
        text(batch, atlas, slot.name(), x, centered_y, name_color);
        return;
    };
    let sub = if joinable { v::SUBTITLE } else { v::GRAY };
    let date_x = cx + cw - v::STAR_AREA - atlas.font.width(release.date);
    text(batch, atlas, release.date, date_x, centered_y, sub);
    match release.title {
        None => text(batch, atlas, release.name, x, centered_y, name_color),
        Some(title) => {
            text(batch, atlas, release.name, x, cy + (ch / 2.0).trunc() - 9.0 - 1.0, name_color);
            // A long update name is cut before the date.
            let first = batch.vertices.len();
            text(batch, atlas, title, x, cy + (ch / 2.0).trunc() + 1.0, sub);
            batch.clip_since(first, x, cy, date_x - v::SLOT_MARGIN, cy + ch);
        }
    }
}

fn setting_row(game: &crate::game::Game<'_>, batch: &mut HudBatch, item: usize, r: [f32; 4]) {
    let atlas = &game.gui.atlas;
    let [cx, cy, cw, ch] = r;
    batch.fill(atlas, cx, cy, cw, ch, rgba(0, 0, 0, 0x80));
    let (value, color) = v::setting_value(item);
    let offset = atlas.font.width(value) + 2.0;
    let y = cy + (ch / 2.0).trunc() - 4.0;
    let first = batch.vertices.len();
    text(batch, atlas, v::SETTINGS[item], cx + v::SLOT_MARGIN, y, 0xAAAAAA);
    batch.clip_since(first, cx, cy, cx + cw - offset - 4.0, cy + ch);
    text(batch, atlas, value, cx + cw - offset, y, color);
}

fn popup(game: &crate::game::Game<'_>, batch: &mut HudBatch, size: [f32; 2], mouse: [f32; 2]) {
    let atlas = &game.gui.atlas;
    let [w, h] = size;
    // Screen.extractTransparentBackground.
    batch.fill(atlas, 0.0, 0.0, w, h, rgba(0x10, 0x10, 0x10, 0xC8));
    let p = v::popup(size);
    let [px, py, pw, ph] = p.panel;
    ui::nine_slice(batch, atlas, "gui/sprites/popup/background", px - 6.0, py - 6.0, pw + 12.0, ph + 12.0, u32::MAX);
    let title = "Report issues";
    let tw = atlas.font.width(title);
    text(batch, atlas, title, px + (pw / 2.0).trunc() - (tw / 2.0).trunc(), py + 5.0, v::ACCENT);
    let cross = if v::inside(p.cross, mouse) {
        "gui/sprites/widget/cross_button_highlighted"
    } else {
        "gui/sprites/widget/cross_button"
    };
    ui::blit_sprite(batch, atlas, cross, p.cross[0], p.cross[1], p.cross[2], p.cross[3], u32::MAX);
    for ((label, active), r) in v::REPORT.iter().zip(&p.buttons) {
        ui::button(batch, atlas, r[0], r[1], r[2], r[3], &Text::literal(*label), *active, v::inside(*r, mouse));
    }
}

/// Over the ping icon: the status, then the version it would join with and
/// the server's own, over the game's tooltip.
fn ping_tooltip(game: &crate::game::Game<'_>, batch: &mut HudBatch, size: [f32; 2], mouse: [f32; 2]) {
    let Some(s) = multiplayer(game) else { return };
    let Some(hover) = rows::ping_hover(s, size, mouse) else { return };
    let target = v::name_of(m::target_for(&hover.address));
    let protocol = m::pinged(&hover.address).map_or("?".to_owned(), |p| p.to_string());
    let server = hover.version.unwrap_or_default();
    let lines = [
        hover.status,
        Text::literal(format!("Target version: {target}")),
        Text::literal(format!("Server version: {server} ({protocol})")),
    ];
    ui::tooltip(batch, &game.gui.atlas, &lines, (mouse[0], mouse[1]));
}
