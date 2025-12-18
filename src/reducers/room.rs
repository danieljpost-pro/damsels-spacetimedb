//! Additional room reducers.
//!
//! Main room operations (create, join, leave) are in player.rs.
//! This module contains additional room management operations.

use spacetimedb::{reducer, ReducerContext};

use crate::models::player::player;
use crate::models::room::{room, room_invitation, RoomInvitation};
use crate::models::enums::RoomInvitationStatus;
use crate::reducers::auth::require_user;

/// Helper: Get a player that belongs to the authenticated user.
fn get_user_player(ctx: &ReducerContext, player_id: u64) -> Result<crate::models::player::Player, String> {
    let user = require_user(ctx)?;
    
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("Player does not belong to you".to_string());
    }
    
    Ok(player)
}

/// Revoke an invitation (owner only).
#[reducer]
pub fn revoke_room_invitation(
    ctx: &ReducerContext,
    player_id: u64,
    invitation_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;

    let invitation = ctx.db.room_invitation().id().find(&invitation_id)
        .ok_or("Invitation not found")?;

    // Check if player owns the room this invitation is for
    let room = ctx.db.room().id().find(&invitation.room_id)
        .ok_or("Room not found")?;
    
    if room.owner_id != player.id {
        return Err("Only the room owner can revoke invitations".to_string());
    }

    if invitation.status != RoomInvitationStatus::Active {
        return Err("Invitation is not active".to_string());
    }

    ctx.db.room_invitation().id().update(RoomInvitation {
        status: RoomInvitationStatus::Revoked,
        ..invitation
    });

    log::info!("Invitation {} revoked by player {}", invitation_id, player.id);
    Ok(())
}
