// 26.3's accept_teleportation carries where the client landed. Older releases took
// the teleport id alone and then, as vanilla's handleMovePlayer did, a
// move_player_pos_rot at that spot (no flags). Without that move an anticheat that
// ties a teleport to the ping sent just before it (Grim: the move must come while
// that ping's pong is the last one received) never sees the teleport accepted: it
// drops every move and sets the player back every few seconds.
fn older_servers_teleport(
    session: &Session,
    state: State,
    id: i32,
    payload: &[u8],
) -> std::ops::ControlFlow<Result<Option<(i32, Vec<u8>)>>> {
    use crate::ids::play::serverbound as sb;
    use std::ops::ControlFlow::{Break, Continue};
    if state != State::Play || session.version == super::ServerVersion::Current {
        return Continue(());
    }
    // 26.3's sign_update is the position, the four lines, then the side (SignTextSlot:
    // VarInt 1 front, else back). Every release before it (26.2, 26.1, 1.21.x, 1.20.x)
    // reads the position, isFrontText, then the lines. Sent as is, the server takes the
    // first line's length as the side and loses the frame ("Failed to decode
    // sign_update"); behind PacketEvents (TotemGuard's sign mod check) it reads a
    // letter of a line as the next packet id ("unknown packet id 101") and drops us.
    if id == sb::SIGN_UPDATE {
        let translate = || -> Result<Option<(i32, Vec<u8>)>> {
            let mut r = Reader::new(payload);
            let pos = r.bytes(8)?;
            let lines = [r.string(384)?, r.string(384)?, r.string(384)?, r.string(384)?];
            let front = r.varint()? == 1;
            let mut w = Writer::new();
            w.bytes(pos);
            w.bool(front);
            for line in &lines {
                w.string(line);
            }
            Ok(session
                .version
                .serverbound_id(state, id)
                .map(|mapped| (mapped, w.data)))
        };
        return Break(translate());
    }
    if id != sb::ACCEPT_TELEPORTATION {
        return Continue(());
    }
    let translate = || -> Result<Option<(i32, Vec<u8>)>> {
        let mut r = Reader::new(payload);
        let teleport = r.varint()?;
        let (x, y, z) = (r.f64()?, r.f64()?, r.f64()?);
        let (yaw, pitch) = (r.f32()?, r.f32()?);
        let mut accept = Writer::new();
        accept.varint(teleport);
        // Horizon's own move_player_pos_rot, laid out for the server by the same
        // translation (1.21.1 and older read the flags as an on-ground boolean).
        let mut pos = Writer::new();
        pos.f64(x);
        pos.f64(y);
        pos.f64(z);
        pos.f32(yaw);
        pos.f32(pitch);
        pos.u8(0);
        if let Some((mapped, body)) = serverbound(session, state, sb::MOVE_PLAYER_POS_ROT, &pos.data)? {
            let mut then = Writer::new();
            then.varint(mapped);
            then.bytes(&body);
            session.outgoing.lock().unwrap().push(then.data);
        }
        Ok(session
            .version
            .serverbound_id(state, id)
            .map(|mapped| (mapped, accept.data)))
    };
    Break(translate())
}
