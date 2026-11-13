# Older Servers (Horizon native mod)

[Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3, Rust) için bir
native mod. Horizon'un yerleşik eski sunucu çeviricisini ortam değişkeni gerektirmeden açar ve
Multiplayer ekranına bir sürüm seçici ekler.

*English summary below.*

## Özellikler (2.0.0: ViaFabricPlus arayüzü)

ViaFabricPlus'ın (26.3 dalı) ekranları Horizon'da, aynı yerleşim ve ölçülerle. ViaFabricPlus'ın
kodu kullanılmadı (GPL-3.0): davranışı ve yerleşimi Rust'ta, Horizon'un kendi widget'larıyla
yeniden yazıldı. ViaFabricPlus ile bağlantısı yoktur.

- **1.20.2–26.2 sunucularına gir** (Horizon'un çeviricisinin bildiği sürümler):
  `HORIZON_OLDER_SERVERS=1` gerekmez.
- **Grim'li sunucularda geri atma yok (2.1.0)**: Horizon'un çeviricisi teleport onayından sonra
  eski istemcilerin yolladığı konum paketini üretmiyordu; Grim teleport'u hiç kabul etmeyip
  oyuncuyu 2-3 saniyede bir geri atıyordu. Mod onayın ardından o paketi ekliyor.
- **Tabela paketi**: 2.1.1 `sign_update`'i eski düzene (`isFrontText` bool) kendisi çeviriyordu;
  Nightly 2026-11-13'ten beri Horizon bunu kendi yapıyor, 2.2.0'da moddan çıkarıldı.
- **ViaFabricPlus butonu** (Multiplayer ve Direct Connect, sağ üst, 98×20) → **sürüm seçici**:
  Back, arama, sekmeler (Favorites, Modern, Legacy, Beta, Alpha, Classic, April Fools), her
  sürüm ikonu, güncelleme adı, çıkış tarihi ve ★ (favori) ile; en üstte **Auto Detect**. Alt
  satırda Server lists, Settings, Report issues. Horizon'un bağlanamadığı sürümler gri ve
  seçilemez; Server lists (BetaCraft/ClassiCube) devre dışı.
- **Add/Edit Server**: sağ üstte `Set version` (ya da sabitlenen sürüm) → **Force Version**
  ekranı, altta Reset. Sabitlenen sürüm `servers.json`'a adresle kaydedilir.
- **Öncelik**: sunucuya sabitlenen sürüm > seçilen sürüm > Auto Detect (sunucunun ping'i).
- **Sunucu listesi**: ping ikonunun ipucunda `Target version` ve `Server version (protokol)`.
- **F3**: `[ViaFabricPlus] 2.2.0`, sunucudayken `P: … C: <istemci> S: <sunucu>`.
- **Settings**: üç butonun yeri (None / Left; Top / Right; Top / Left; Bottom / Right; Bottom),
  seçilen sürümü sakla, sunucu sürümünü göster.
- **1.21.11 eşya düzeltmesi**: EitherHolder bileşenleri upstream'e girdi
  (horizon-rust/horizon#10038); 2.1.0'dan beri Horizon'un kendisinde, modda değil.

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
python tools/package.py --install     # target/older_servers-2.2.0.hnmod → horizon-native-mods/
```

Sonra Horizon launcher'ında oyunu native modlarıyla derle.

## Sabitlenmiş sürüm (pin)

Native hook noktası olmayan yerlere dokunduğu için Horizon Nightly 2026-11-13'e (`9f42064`)
sabitlenmiştir. İç erişimlerin hepsi `horizon-mixin.toml`'un başındaki yorumda listelenir:

- `horizon_client_lib::net::older_servers_enabled`, `::older_server` (bağlantı sürümü)
- `horizon_protocol::adapter::translate::serverbound` (teleport onayının ardından konum paketi;
  `teleport.rs`)
- `horizon_client_lib::multiplayer::ping_all` ve `multiplayer.rs` içindeki `older_servers_rows`
  modülü (satırların yeri, Edit ekranındaki adres)
- `horizon_client::game::debug_lines::Game::debug_entry_lines` (F3 satırı)
- `Game`'in özel `title` alanı (screen.rs, mouse.rs)

Yeni bir Nightly bunları taşıyabilir: önce kontrol et, sonra `compatibility.json` ve
`mixins/older_servers/horizon-mixin.toml` içindeki pin'i birlikte taşı. Teleport ve tabela
düzeltmeleri upstream'e girince `teleport.rs` kaldırılabilir.

## Test

- `horizon-mixins plan` `f319701` üzerinde temiz; `cargo check` temiz; modun 21 birim testi
  geçiyor (seçim, ayarlar, yerleşim, panel olayları, adres normalleştirme, etiketler, sprite
  dilimleme).
- Çevirici, Paper 1.21.11'e karşı gerçek bağlantıyla denendi (horizon-bots ile): yürüme,
  `/tp` sonrası, ölüp yeniden doğma, Nether/End geçişi; 1867 chunk'ta 0 çeviri hatası.
- **"1.21.11 sunucularda hareket edemiyorum"** (2.1.0'da çözüldü): Grim'li sunuculara özgüydü.
  Yerelde Paper 1.21.11 + Grim 2.3.73 ile üretildi; fix'le yürüme/koşma/zıplama ve `/tp`
  sonrasında istemci ile sunucu konumu aynı, Grim ihlali ve geri atma yok.
- **Tabela paketi** (2.1.1): Paper + PacketEvents 2.11.2 + TotemGuard 2.1.2 ile A/B: eski build
  girişten hemen sonra atıldı, yenisi bağlı kaldı.
- 2.1.1, Nightly 2026-11-07 `2ebf00a` üzerinde 9 başka native modla birlikte plan + performance
  build ile derlendi ve oyunda denendi.
- 2.2.0, Nightly 2026-11-13 `9f42064` üzerinde 10 başka native modla birlikte plan + release build
  ile derlendi; eski sunucuya bağlanma bu sürümde oyunda yeniden denenmedi.

## English

A native mod for Horizon that turns on its older-servers adapter without environment variables
(1.20.2 to 26.2), adds a version panel to the Multiplayer screen, per-server pinned
versions (Edit/Add Server), version tags in the server list, an F3 line and settings in
Options > Mods. It also fixes a gap in the adapter: after accepting a teleport it sends the
position packet older clients send (without it Grim never accepts the teleport and keeps pulling
the player back). Up to 2.1.1 it also wrote `sign_update` in the pre-26.3 layout; Horizon does
that itself since Nightly 2026-11-13, so 2.2.0 dropped it. It is pinned to Horizon `9f42064`
(Nightly 2026-11-13); see the list above for every internal access.

Mojang, Microsoft ve Horizon projesiyle bağlantısı yoktur.

## Lisans

MIT, bkz. [LICENSE](LICENSE).
