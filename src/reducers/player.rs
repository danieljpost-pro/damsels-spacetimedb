//! Player-related reducers.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::player::{player, Player};
use crate::models::room::{room, room_member, Room, RoomMember};
use crate::models::enums::PlayerRole;
use crate::utils::generate_room_code;

/// Register a new player or update existing player's last_seen.
/// Returns the player's id for client reference.
#[reducer]
pub fn register_player(ctx: &ReducerContext, username: String) -> Result<(), String> {
    // Check if player already exists with this identity
    if let Some(existing) = ctx.db.player().identity().find(&ctx.sender) {
        // Update last_seen and return existing player
        ctx.db.player().id().update(Player {
            last_seen: ctx.timestamp,
            ..existing
        });
        log::info!("Player reconnected: {:?} ({})", ctx.sender, username);
        return Ok(());
    }

    // Check if username is taken by another player
    if ctx.db.player().username().find(&username).is_some() {
        return Err("Username already taken".to_string());
    }

    let now = ctx.timestamp;
    ctx.db.player().insert(Player {
        id: 0,
        identity: ctx.sender,
        username,
        xp: 0,
        created_at: now,
        last_seen: now,
    });

    log::info!("Player registered: {:?}", ctx.sender);
    Ok(())
}

/// Sign in a player and auto-create/join their personal room.
/// If the player already has a room they own, rejoin it.
/// Otherwise, create a new room with the specified role.
#[reducer]
pub fn sign_in_with_room(ctx: &ReducerContext, username: String, role: PlayerRole) -> Result<(), String> {
    // ActivityAdmin appears as Observer to other players
    #[cfg(feature = "dev")]
    let role = if matches!(role, PlayerRole::ActivityAdmin) {
        PlayerRole::Observer
    } else {
        role
    };

    let now = ctx.timestamp;
    
    // Register or get existing player
    let player = if let Some(existing) = ctx.db.player().identity().find(&ctx.sender) {
        // Update last_seen
        ctx.db.player().id().update(Player {
            last_seen: now,
            ..existing.clone()
        });
        existing
    } else {
        // Check if username is taken
        if ctx.db.player().username().find(&username).is_some() {
            return Err("Username already taken".to_string());
        }
        
        // Create new player
        ctx.db.player().insert(Player {
            id: 0,
            identity: ctx.sender,
            username: username.clone(),
            xp: 0,
            created_at: now,
            last_seen: now,
        })
    };
    
    // Check if player already owns a room
    if let Some(existing_room) = ctx.db.room().owner_id().filter(&player.id).next() {
        // Check if already a member
        let is_member = ctx.db.room_member()
            .room_id()
            .filter(&existing_room.id)
            .find(|m| m.player_id == player.id)
            .is_some();
        
        if !is_member {
            // Rejoin their own room
            ctx.db.room_member().insert(RoomMember {
                id: 0,
                room_id: existing_room.id,
                player_id: player.id,
                role,
                joined_at: now,
            });
        }
        
        log::info!("Player {} rejoined their room {}", player.id, existing_room.code);
        return Ok(());
    }
    
    // Create a new room for this player
    let code = generate_room_code(ctx);
    let room = ctx.db.room().insert(Room {
        id: 0,
        code: code.clone(),
        owner_id: player.id,
        created_at: now,
    });
    
    // Add player as first member
    ctx.db.room_member().insert(RoomMember {
        id: 0,
        room_id: room.id,
        player_id: player.id,
        role,
        joined_at: now,
    });
    
    log::info!("Player {} created room {}", player.id, code);
    Ok(())
}

