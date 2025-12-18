//! Lifecycle reducers for client connection events.

use spacetimedb::{reducer, ReducerContext};

use crate::models::user::{user, User};

/// Called when a client connects.
#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    log::info!("Client connected: {:?}", ctx.sender);
}

/// Called when a client disconnects.
#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    log::info!("Client disconnected: {:?}", ctx.sender);
    
    // Update last_seen for the user
    if let Some(existing_user) = ctx.db.user().identity().find(&ctx.sender) {
        ctx.db.user().id().update(User {
            last_seen: ctx.timestamp,
            ..existing_user
        });
    }
}
