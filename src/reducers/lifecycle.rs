//! Lifecycle reducers for client connection events.

use spacetimedb::{reducer, ReducerContext};

use crate::models::player::{player, Player};

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

