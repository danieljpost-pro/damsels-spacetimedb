//! Player-related reducers for room operations.
//!
//! All room operations require:
//! 1. An authenticated User (via login_user)
//! 2. A selected Player ID that belongs to that User

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::player::{player, Player};
use crate::models::room::{room, room_member, room_invitation, Room, RoomMember, RoomInvitation};
use crate::models::enums::{PlayerRole, RoomInvitationStatus};
use crate::reducers::auth::require_user;
use crate::utils::generate_room_code;

/// Helper: Get a player that belongs to the authenticated user.
fn get_user_player(ctx: &ReducerContext, player_id: u64) -> Result<Player, String> {
    let user = require_user(ctx)?;
    
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("Player does not belong to you".to_string());
    }
    
    Ok(player)
}

/// Create a new room with the specified player as owner.
#[reducer]
pub fn create_room(
    ctx: &ReducerContext,
    player_id: u64,
    room_name: String,
    role: PlayerRole,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Check if player is already in a room
    let existing_membership = ctx.db.room_member()
        .player_id()
        .filter(&player.id)
        .find(|_| true);
    
    if existing_membership.is_some() {
        return Err("Player is already in a room".to_string());
    }
    
    let code = generate_room_code(ctx);
    let display_name = if room_name.trim().is_empty() {
        format!("{}'s Room", player.username)
    } else {
        room_name
    };

    let new_room = ctx.db.room().insert(Room {
        id: 0,
        code: code.clone(),
        name: display_name.clone(),
        owner_id: player.id,
        is_open: true,
        created_at: ctx.timestamp,
    });
    
    ctx.db.room_member().insert(RoomMember {
        id: 0,
        room_id: new_room.id,
        player_id: player.id,
        role,
        joined_at: ctx.timestamp,
    });
    
    log::info!("Player {} created room {} ({})", player.id, display_name, code);
    Ok(())
}

/// Join an existing room by code.
#[reducer]
pub fn join_room(
    ctx: &ReducerContext,
    player_id: u64,
    room_code: String,
    role: PlayerRole,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let target_room = ctx.db.room().code().find(&room_code)
        .ok_or("Room not found")?;

    if !target_room.is_open {
        return Err("Room is closed".to_string());
    }
    
    // Check if already a member
    let already_member = ctx.db.room_member()
        .room_id()
        .filter(&target_room.id)
        .any(|m| m.player_id == player.id);

    if already_member {
        return Err("Already in this room".to_string());
    }

    ctx.db.room_member().insert(RoomMember {
        id: 0,
        room_id: target_room.id,
        player_id: player.id,
        role,
        joined_at: ctx.timestamp,
    });

    log::info!("Player {} joined room {}", player.id, room_code);
    Ok(())
}

/// Accept an invitation and join a room.
#[reducer]
pub fn accept_invitation(
    ctx: &ReducerContext,
    player_id: u64,
    invitation_token: String,
    role: PlayerRole,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let invitation = ctx.db.room_invitation().token().find(&invitation_token)
        .ok_or("Invalid invitation token")?;

    if invitation.status != RoomInvitationStatus::Active {
        return Err("Invitation is no longer valid".to_string());
    }

    // Check if restricted to specific player name
    if let Some(ref required_username) = invitation.for_username {
        if &player.username != required_username {
            return Err("This invitation is for a different player".to_string());
        }
    }

    let target_room = ctx.db.room().id().find(&invitation.room_id)
        .ok_or("Room no longer exists")?;

    if !target_room.is_open {
        return Err("Room is closed".to_string());
    }

    // Check if already a member
    let already_member = ctx.db.room_member()
        .room_id()
        .filter(&target_room.id)
        .any(|m| m.player_id == player.id);

    if already_member {
        return Err("Already in this room".to_string());
    }

    // Mark invitation as used
    ctx.db.room_invitation().id().update(RoomInvitation {
        status: RoomInvitationStatus::Used,
        accepted_by: Some(player.id),
        ..invitation
    });

    // Join the room
    ctx.db.room_member().insert(RoomMember {
        id: 0,
        room_id: target_room.id,
        player_id: player.id,
        role,
        joined_at: ctx.timestamp,
    });

    log::info!("Player {} accepted invitation and joined room {}", player.id, target_room.id);
    Ok(())
}

/// Leave a room.
#[reducer]
pub fn leave_room(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Find the player's room membership
    let membership = ctx.db.room_member()
        .player_id()
        .filter(&player.id)
        .find(|_| true)
        .ok_or("Not in any room")?;
    
    let room = ctx.db.room().id().find(&membership.room_id)
        .ok_or("Room not found")?;
    
    // Remove from room
    ctx.db.room_member().id().delete(&membership.id);
    
    // If owner, close the room
    if room.owner_id == player.id {
        ctx.db.room().id().update(Room {
            is_open: false,
            ..room
        });
        log::info!("Player {} left and closed room {}", player.id, membership.room_id);
    } else {
        log::info!("Player {} left room {}", player.id, membership.room_id);
    }
    
    Ok(())
}

/// Change role within current room.
#[reducer]
pub fn change_role(ctx: &ReducerContext, player_id: u64, new_role: PlayerRole) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let membership = ctx.db.room_member()
        .player_id()
        .filter(&player.id)
        .find(|_| true)
        .ok_or("Not in any room")?;
    
    ctx.db.room_member().id().update(RoomMember {
        role: new_role,
        ..membership
    });
    
    log::info!("Player {} changed role to {:?}", player.id, new_role);
    Ok(())
}

/// Create an invitation for a room (owner only).
/// The invitation token can be retrieved by subscribing to room_invitation table.
#[reducer]
pub fn create_room_invitation(
    ctx: &ReducerContext,
    player_id: u64,
    for_username: Option<String>,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Find player's room where they are owner
    let room = ctx.db.room().owner_id().filter(&player.id).find(|r| r.is_open)
        .ok_or("You don't own an open room")?;
    
    // Generate invitation token
    let token = generate_room_code(ctx);
    
    ctx.db.room_invitation().insert(RoomInvitation {
        id: 0,
        room_id: room.id,
        token: token.clone(),
        created_by: player.id,
        for_username,
        status: RoomInvitationStatus::Active,
        accepted_by: None,
        created_at: ctx.timestamp,
    });
    
    log::info!("Player {} created invitation {} for room {}", player.id, token, room.id);
    Ok(())
}

/// Close a room (owner only).
#[reducer]
pub fn close_room(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let room = ctx.db.room().owner_id().filter(&player.id).find(|r| r.is_open)
        .ok_or("You don't own an open room")?;
    
    ctx.db.room().id().update(Room {
        is_open: false,
        ..room
    });
    
    // Invalidate all active invitations
    for inv in ctx.db.room_invitation().room_id().filter(&room.id) {
        if inv.status == RoomInvitationStatus::Active {
            ctx.db.room_invitation().id().update(RoomInvitation {
                status: RoomInvitationStatus::Revoked,
                ..inv
            });
        }
    }
    
    log::info!("Player {} closed room {}", player.id, room.id);
    Ok(())
}
