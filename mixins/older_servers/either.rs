// 1.21.11 sends damage_type, instrument, provides_trim_material,
// chicken/variant and zombie_nautilus/variant as EitherHolders (a flag, then the
// holder or its ResourceKey), as it does jukebox_playable, which Horizon already
// reads: the flag goes, a holder reads on as 26.3's, a key drops the component.
// Upstream: horizon-rust/horizon#9752.
fn older_servers_either_holder(kind: i32, r: &mut Reader) -> std::ops::ControlFlow<Result<Older>> {
    use std::ops::ControlFlow::{Break, Continue};
    // damage_type 8, instrument 63, provides_trim_material 64, chicken/variant 104,
    // zombie_nautilus/variant 106 (26.3's ids).
    if !matches!(kind, 8 | 63 | 64 | 104 | 106)
        || !crate::adapter::ids::quirks().either_holder_song_and_tag_resistance
    {
        return Continue(());
    }
    let read = |r: &mut Reader| -> Result<Older> {
        if r.bool()? {
            Ok(Older::Pinned)
        } else {
            r.string(32767)?;
            Ok(Older::Dropped)
        }
    };
    Break(read(r))
}
