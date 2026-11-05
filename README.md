# Older Servers (Horizon native mod)

[Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3, Rust) için bir
native mod. Horizon'un yerleşik eski sunucu çeviricisini ortam değişkeni gerektirmeden açar ve
Multiplayer ekranına bir sürüm seçici ekler.

*English summary below.*

## Özellikler

- **1.21.10, 1.21.11, 26.1 ve 26.2 sunucularına gir**: `HORIZON_OLDER_SERVERS=1` gerekmez.
- **Sürüm paneli** (Multiplayer ekranı): köşedeki `Sürüm: …` butonuna tıklayınca vanilla
  görünümlü bir liste açılır. Listede `Otomatik (sunucuya göre)` / `Varsayılan (26.3)` ve
  desteklenen tüm sürümler (en yeniden eskiye) bulunur. Seçmek için tıkla ya da ↑/↓ ve Enter
  kullan; kapatmak için Esc, panelin dışına tıkla ya da `Tamam`'a bas. Tekerlekle kaydırılır.
  Panelin altındaki `Otomatik algılama: AÇIK/KAPALI` butonu algılamayı açıp kapatır.
- **Sunucu başına sürüm**: Edit Server / Add Server ekranında adres alanının yanındaki buton o
  sunucuya bir sürüm sabitler (sol tık ileri, sağ tık geri). Sabitlenen sürüm
  `servers.json` dosyasına adresle (küçük harf, varsayılan port `:25565`) kaydedilir.
- **Öncelik**: sunucuya sabitlenen sürüm > listeden seçilen sürüm > otomatik algılama (ping).
  Otomatik algılama kapalıysa ve hiçbir sürüm seçilmemişse istemci 26.3 konuşur.
- **Listede sürüm etiketi**: sunucu adının sağında hangi sürümle bağlanılacağı yazar.
  Sarı `→ 1.21.11` çeviri yapılacak demektir, yeşil `26.3` çeviri yok, kırmızı ise sunucunun
  sürümü seçilenle uyuşmuyor ya da desteklenmiyor (`✕ 47`).
- **F3 satırı**: bir dünyadayken `Older Servers: 1.21.11 (774), auto` (kaynak: `per server`,
  `picked`, `auto` ya da `default`).
- **Ayarlar** (Options > Mods > Older Servers): otomatik algılama, butonun köşesi, butonu gizle,
  liste etiketleri, F3 satırı ve sürüm seçimi.
- **1.21.11 eşya düzeltmesi**: 1.21.11 `instrument`, `provides_trim_material`, `damage_type`,
  `chicken/variant` ve `zombie_nautilus/variant` bileşenlerini EitherHolder olarak gönderir;
  çevirici bunları 26.3'ün düz holder'ı gibi okuyup paketi düşürüyordu (keçi boruları, zırh
  süsleri, yumurtalar…). Upstream: horizon-rust/horizon#9752.

Butonlar ve liste Minecraft'ın kendi `widget/button`, `widget/scroller` sprite'larıyla çizilir;
modun içinde görsel dosyası yoktur.

## Kayıtlı veriler

`horizon-mods/.horizon-mod-data/older_servers/` altında:

| Anahtar | İçerik |
| --- | --- |
| `version` | Listeden seçilen sürüm (1.0.0'daki anahtar, aynen okunur) |
| `auto_detect`, `button_corner`, `hide_button`, `list_tags`, `debug_line` | Ayarlar |
| `servers.json` | Sunucu başına sabitlenen sürümler (`{"play.example.com:25565": "1.21.11"}`) |

## Kurulum

```sh
python tools/package.py --install     # target/older_servers-1.1.0.hnmod → horizon-native-mods/
```

Sonra Horizon launcher'ında oyunu native modlarıyla derle.

## Sabitlenmiş sürüm (pin)

Native hook noktası olmayan yerlere dokunduğu için Horizon Nightly 2026-11-05'e (`f319701`)
sabitlenmiştir. İç erişimlerin hepsi `horizon-mixin.toml`'un başındaki yorumda listelenir:

- `horizon_client_lib::net::older_servers_enabled`, `::older_server` (bağlantı sürümü)
- `horizon_protocol::item::older_layout` (1.21.11 EitherHolder bileşenleri)
- `horizon_client_lib::multiplayer::ping_all` ve `multiplayer.rs` içindeki `older_servers_rows`
  modülü (satırların yeri, Edit ekranındaki adres)
- `horizon_client::game::debug_lines::Game::debug_entry_lines` (F3 satırı)
- `Game`'in özel `title` alanı (screen.rs, mouse.rs)

Yeni bir Nightly bunları taşıyabilir: önce kontrol et, sonra `compatibility.json` ve
`mixins/older_servers/horizon-mixin.toml` içindeki pin'i birlikte taşı. Eşya düzeltmesi
upstream'e girince `either.rs` kaldırılabilir.

## Test

- `horizon-mixins plan` `f319701` üzerinde temiz; `cargo check` temiz; modun 21 birim testi
  geçiyor (seçim, ayarlar, yerleşim, panel olayları, adres normalleştirme, etiketler, sprite
  dilimleme).
- Çevirici, Paper 1.21.11'e karşı gerçek bağlantıyla denendi (horizon-bots ile): yürüme,
  `/tp` sonrası, ölüp yeniden doğma, Nether/End geçişi; 1867 chunk'ta 0 çeviri hatası.
- **"1.21.11 sunucularda hareket edemiyorum"**: protokol tarafında yeniden üretilemedi
  (serverbound id'leri, teleport onayı, MovePlayer/PlayerInput düzenleri, entity data ve
  attribute eşlemeleri, GameEvent id'leri tek tek karşılaştırıldı). Sorun istemci tarafında
  görünüyor; tekrar ederse istemcinin `logs/latest.log` dosyası ve takıldığı an gerekli.

## English

A native mod for Horizon that turns on its older-servers adapter without environment variables
(1.21.10, 1.21.11, 26.1, 26.2), adds a version panel to the Multiplayer screen, per-server pinned
versions (Edit/Add Server), version tags in the server list, an F3 line, settings in
Options > Mods, and a fix for 1.21.11's EitherHolder item components. It is pinned to Horizon
`f319701`; see the list above for every internal access.

Mojang, Microsoft ve Horizon projesiyle bağlantısı yoktur.

## Lisans

MIT, bkz. [LICENSE](LICENSE).
