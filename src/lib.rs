//! Damsels - SpacetimeDB Game Backend
//!
//! A session-based, real-time multiplayer game with an Activity Tree
//! progression system.

pub mod models;

// Development-only admin module (conditionally compiled)
#[cfg(feature = "dev")]
pub mod admin;

// Re-export models for SpacetimeDB table registration
pub use models::*;

use spacetimedb::{reducer, ReducerContext, Table};

// Import table traits for ctx.db access (internal, not part of public API)
use crate::models::player::player;
use crate::models::room::{room, room_member};

// =============================================================================
// Lifecycle Reducers
// =============================================================================

/// Called when a client connects.
#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    log::info!("Client connected: {:?}", ctx.sender);
}

/// Called when a client disconnects.
#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    log::info!("Client disconnected: {:?}", ctx.sender);
    
    // Update last_seen for the player
    if let Some(player) = ctx.db.player().identity().find(&ctx.sender) {
        ctx.db.player().id().update(Player {
            last_seen: ctx.timestamp,
            ..player
        });
    }
}

// =============================================================================
// Player Reducers
// =============================================================================

/// Register a new player or return existing player.
#[reducer]
pub fn register_player(ctx: &ReducerContext, username: String) -> Result<(), String> {
    // Check if player already exists
    if ctx.db.player().identity().find(&ctx.sender).is_some() {
        return Err("Player already registered".to_string());
    }

    // Check if username is taken
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

// =============================================================================
// Room Reducers
// =============================================================================

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

    // Generate a room code (simple implementation)
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

// =============================================================================
// Helper Functions
// =============================================================================

/// Generate a 5-character room code.
/// Uses alphanumeric characters excluding ambiguous ones (0, O, I, L, 1).
fn generate_room_code(ctx: &ReducerContext) -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let micros = ctx.timestamp.to_duration_since_unix_epoch().unwrap_or_default().as_micros();
    
    // Generate 5 characters from timestamp entropy
    let mut code = String::with_capacity(5);
    let mut n = micros;
    for _ in 0..5 {
        let idx = (n % CHARS.len() as u128) as usize;
        code.push(CHARS[idx] as char);
        n /= CHARS.len() as u128;
    }
    code
}
