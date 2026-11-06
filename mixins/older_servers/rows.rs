// Multiplayer ekranının iç düzenine tek erişim noktası: sunucu satırlarının
// nerede çizildiği (sürüm etiketi için) ve Edit/Add Server ekranındaki adres.
// Liste çizimine (`Screen::draw`) iç enjeksiyon kabul edilmediği için
// (`E_UNSUPPORTED_CONTROL_FLOW`) etiket `screen_render_post`'ta, buradaki
// yerlerle çizilir. `horizon_client::multiplayer` içine `older_servers_rows`
// olarak eklenir: özel `list_area`, `rows`, `ROW_HEIGHT`'i okuyabilir. Pin
// taşınmadan önce `Screen::draw`'daki satır yerleşimiyle karşılaştırılmalı.
use super::*;

/// Görünen bir sunucu satırı: adresi, adı, adın sol üstü ve etiketin
/// geçmemesi gereken sağ sınır (sağdaki oyuncu sayısının solu).
pub struct VisibleRow {
    pub address: String,
    pub name: String,
    pub at: [f32; 2],
    pub limit: f32,
}

/// Liste ekranında görünen kayıtlı sunucu satırları, `Screen::draw`'ın yerleşimiyle.
pub fn visible(screen: &Screen, size: [f32; 2]) -> Vec<VisibleRow> {
    let Screen::List {
        servers,
        scroll,
        lan,
        ..
    } = screen
    else {
        return Vec::new();
    };
    let (w, h) = (size[0], size[1]);
    let games = lan.games().len();
    let (top, bottom, left) = list_area(w, h);
    let content = content_height(servers.len(), games);
    let mut scroll = *scroll;
    scroll.clamp(top, bottom, content);
    let mut out = Vec::new();
    for (i, row) in rows(servers.len(), games).enumerate() {
        let Row::Server(k) = row else { continue };
        let y = top + 2.0 - scroll.amount.trunc() + i as f32 * ROW_HEIGHT;
        // Adın satırı (y + 3 .. y + 12) listenin içinde kalmalı.
        if y + 3.0 < top || y + 12.0 > bottom {
            continue;
        }
        let (cx, cy) = (left + 2.0, y + 2.0);
        out.push(VisibleRow {
            address: servers[k].ip.clone(),
            name: servers[k].name.clone(),
            at: [cx + 35.0, cy + 1.0],
            limit: cx + ROW_WIDTH - 4.0 - 15.0 - 45.0,
        });
    }
    out
}

/// Edit/Add Server ekranında yazılı adres.
pub fn edit_address(screen: &Screen) -> Option<&str> {
    match screen {
        Screen::Edit { address, .. } => Some(address.as_str()),
        _ => None,
    }
}

/// The ping icon under the pointer: the server's address, the game's own
/// tooltip line for it, and the version its status named.
pub struct PingHover {
    pub address: String,
    pub status: crate::text::Text,
    pub version: Option<String>,
}

pub fn ping_hover(screen: &Screen, size: [f32; 2], cursor: [f32; 2]) -> Option<PingHover> {
    let Screen::List {
        servers,
        pings,
        lan,
        scroll,
        ..
    } = screen
    else {
        return None;
    };
    let size = (size[0].ceil(), size[1].ceil());
    let (top, bottom, _) = list_area(size.0, size.1);
    let mut scroll = *scroll;
    scroll.clamp(top, bottom, content_height(servers.len(), lan.games().len()));
    let pings = pings.lock().unwrap();
    let at = (cursor[0], cursor[1]);
    let status = ping_tooltip(servers, &pings, size, scroll.amount, at)?;
    let Row::Server(i) = row_at(servers.len(), 0, at, size, scroll.amount)? else {
        return None;
    };
    let version = match pings.get(&servers[i].ip) {
        Some(Ping::Ok { version, .. }) => Some(version.clone()),
        Some(Ping::Incompatible { version, .. }) => Some(version.plain()),
        _ => None,
    };
    Some(PingHover {
        address: servers[i].ip.clone(),
        status,
        version,
    })
}
