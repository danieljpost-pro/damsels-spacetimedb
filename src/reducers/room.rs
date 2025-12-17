//! Room-related reducers.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::player::player;
use crate::models::room::{room, room_member, Room, RoomMember};
use crate::models::enums::PlayerRole;
use crate::utils::generate_room_code;

/// Create a new room.
#[reducer]
pub fn create_room(ctx: &ReducerContext, role: PlayerRole) -> Result<(), String> {
    // ActivityAdmin appears as Observer to other players
    #[cfg(feature = "dev")]
    let role = if matches!(role, PlayerRole::ActivityAdmin) {
        PlayerRole::Observer
    } else {
        role
    };

    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    // Generate a room code
    let code = generate_room_code(ctx);

    let now = ctx.timestamp;
    let room = ctx.db.room().insert(Room {
        id: 0,
        code: code.clone(),
        owner_id: player.id,
        created_at: now,
    });

    // Add creator as first member
    ctx.db.room_member().insert(RoomMember {
        id: 0,
        room_id: room.id,
        player_id: player.id,
        role,
        joined_at: now,
    });

    log::info!("Room created: {} by player {}", code, player.id);
    Ok(())
}

/// Join an existing room by code.
#[reducer]
pub fn join_room(ctx: &ReducerContext, room_code: String, role: PlayerRole) -> Result<(), String> {
    // ActivityAdmin appears as Observer to other players
    #[cfg(feature = "dev")]
    let role = if matches!(role, PlayerRole::ActivityAdmin) {
        PlayerRole::Observer
    } else {
        role
    };

    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    let room = ctx.db.room().code().find(&room_code)
        .ok_or("Room not found")?;

    // Check if already a member
    let existing = ctx.db.room_member()
        .room_id()
        .filter(&room.id)
        .find(|m| m.player_id == player.id);

    if existing.is_some() {
        return Err("Already in this room".to_string());
    }

    ctx.db.room_member().insert(RoomMember {
        id: 0,
        room_id: room.id,
        player_id: player.id,
        role,
        joined_at: ctx.timestamp,
    });

    log::info!("Player {} joined room {}", player.id, room_code);
    Ok(())
}

/// Change role within a room.
#[reducer]
pub fn change_role(ctx: &ReducerContext, room_id: u64, new_role: PlayerRole) -> Result<(), String> {
    // ActivityAdmin appears as Observer to other players
    #[cfg(feature = "dev")]
    let new_role = if matches!(new_role, PlayerRole::ActivityAdmin) {
        PlayerRole::Observer
    } else {
        new_role
    };

    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    let member = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .find(|m| m.player_id == player.id)
        .ok_or("Not a member of this room")?;

    ctx.db.room_member().id().update(RoomMember {
        role: new_role,
        ..member
    });

    log::info!("Player {} changed role to {:?} in room {}", player.id, new_role, room_id);
    Ok(())
}

/// Leave a room.
#[reducer]
pub fn leave_room(ctx: &ReducerContext, room_id: u64) -> Result<(), String> {
    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    let member = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .find(|m| m.player_id == player.id)
        .ok_or("Not a member of this room")?;

    ctx.db.room_member().id().delete(&member.id);

    // Check if room is now empty
    let remaining = ctx.db.room_member().room_id().filter(&room_id).count();
    if remaining == 0 {
        // Delete the room
        ctx.db.room().id().delete(&room_id);
        log::info!("Room {} deleted (empty)", room_id);
    }

    log::info!("Player {} left room {}", player.id, room_id);
    Ok(())
}

