# Older Servers (Horizon native mod)

[Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3, Rust) için bir
native mod. Horizon'un yerleşik eski sunucu çeviricisini ortam değişkeni gerektirmeden açar ve
Multiplayer ekranına bir sürüm seçici ekler.

*English summary below.*

## Özellikler (2.0.0: ViaFabricPlus arayüzü)

ViaFabricPlus'ın (26.3 dalı) ekranları Horizon'da, aynı yerleşim ve ölçülerle. ViaFabricPlus'ın
kodu kullanılmadı (GPL-3.0): davranışı ve yerleşimi Rust'ta, Horizon'un kendi widget'larıyla
yeniden yazıldı. ViaFabricPlus ile bağlantısı yoktur.

- **1.21.9–26.2 sunucularına gir**: `HORIZON_OLDER_SERVERS=1` gerekmez.
- **ViaFabricPlus butonu** (Multiplayer ve Direct Connect, sağ üst, 98×20) → **sürüm seçici**:
  Back, arama, sekmeler (Favorites, Modern, Legacy, Beta, Alpha, Classic, April Fools), her
  sürüm ikonu, güncelleme adı, çıkış tarihi ve ★ (favori) ile; en üstte **Auto Detect**. Alt
  satırda Server lists, Settings, Report issues. Horizon'un bağlanamadığı sürümler gri ve
  seçilemez; Server lists (BetaCraft/ClassiCube) devre dışı.
- **Add/Edit Server**: sağ üstte `Set version` (ya da sabitlenen sürüm) → **Force Version**
  ekranı, altta Reset. Sabitlenen sürüm `servers.json`'a adresle kaydedilir.
- **Öncelik**: sunucuya sabitlenen sürüm > seçilen sürüm > Auto Detect (sunucunun ping'i).
- **Sunucu listesi**: ping ikonunun ipucunda `Target version` ve `Server version (protokol)`.
- **F3**: `[ViaFabricPlus] 2.0.1`, sunucudayken `P: … C: <istemci> S: <sunucu>`.
- **Settings**: üç butonun yeri (None / Left; Top / Right; Top / Left; Bottom / Right; Bottom),
  seçilen sürümü sakla, sunucu sürümünü göster.
- **1.21.11 eşya düzeltmesi**: EitherHolder bileşenleri (upstream'e girdi: horizon-rust/horizon#10038).

## Kayıtlı veriler

`horizon-mods/.horizon-mod-data/older_servers/` altında:

| Anahtar | İçerik |
| --- | --- |
| `version` | Listeden seçilen sürüm (1.0.0'daki anahtar, aynen okunur) |
| `auto_detect`, `debug_line`, `mp_button`, `add_button`, `direct_button`, `save_selected`, `show_advertised` | Ayarlar |
| `favorites` | Favori sürümler, satır başına bir ad |
| `servers.json` | Sunucu başına sabitlenen sürümler (`{"play.example.com:25565": "1.21.11"}`) |

## Kurulum

```sh
python tools/package.py --install     # target/older_servers-2.0.1.hnmod → horizon-native-mods/
```

Sonra Horizon launcher'ında oyunu native modlarıyla derle.

## Sabitlenmiş sürüm (pin)

Native hook noktası olmayan yerlere dokunduğu için Horizon Nightly 2026-11-05'e (`66f3981`)
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
