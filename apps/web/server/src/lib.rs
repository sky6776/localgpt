use spacetimedb::{reducer, table, Identity, ReducerContext, SpacetimeType, Timestamp};
use std::collections::HashMap;

// ============================================================================
// Tables
// ============================================================================

/// A player in the world
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_y: f32,
    pub online: bool,
    pub last_seen: Timestamp,
}

/// A generated world entity (tree, rock, building, etc.)
#[table(name = world_entity, public)]
pub struct WorldEntity {
    #[primary_key]
    pub id: u64,
    pub entity_type: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_y: f32,
    pub scale: f32,
    pub metadata: String, // JSON for additional properties
}

/// Chat messages
#[table(name = chat_message, public)]
pub struct ChatMessage {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub sender_identity: Identity,
    pub sender_name: String,
    pub message: String,
    pub timestamp: Timestamp,
}

/// World metadata (singleton per world)
#[table(name = world_info, public)]
pub struct WorldInfo {
    #[primary_key]
    pub id: u64,
    pub name: String,
    pub seed: u64,
    pub width: f32,
    pub depth: f32,
    pub created_at: Timestamp,
}

/// Counter for entity IDs
#[table(name = entity_counter)]
pub struct EntityCounter {
    #[primary_key]
    pub id: u8,
    pub count: u64,
}

// ============================================================================
// Lifecycle
// ============================================================================

#[reducer]
pub fn __identity_connected(ctx: &ReducerContext) {
    // Initialize player on connect
    if ctx.db.player().identity().find(ctx.sender).is_none() {
        ctx.db.player().insert(Player {
            identity: ctx.sender,
            name: format!("Player_{}", &ctx.sender.to_hex()[..8]),
            x: 0.0,
            y: 1.0,
            z: 0.0,
            rotation_y: 0.0,
            online: true,
            last_seen: ctx.timestamp,
        });
    } else {
        // Mark existing player as online
        if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
            player.online = true;
            player.last_seen = ctx.timestamp;
            ctx.db.player().identity().update(player);
        }
    }

    // Initialize world if not exists
    if ctx.db.world_info().id().find(0).is_none() {
        let seed = ctx.timestamp.to_micros_since_unix_epoch() as u64;
        ctx.db.world_info().insert(WorldInfo {
            id: 0,
            name: "Generated World".to_string(),
            seed,
            width: 100.0,
            depth: 100.0,
            created_at: ctx.timestamp,
        });

        // Generate initial world
        generate_world(ctx, seed);
    }
}

#[reducer]
pub fn __identity_disconnected(ctx: &ReducerContext) {
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        player.online = false;
        player.last_seen = ctx.timestamp;
        ctx.db.player().identity().update(player);
    }
}

// ============================================================================
// Player Actions
// ============================================================================

#[reducer]
pub fn set_player_name(ctx: &ReducerContext, name: String) {
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        // Limit name length
        let name: String = name.chars().take(32).collect();
        player.name = name;
        ctx.db.player().identity().update(player);
    }
}

#[reducer]
pub fn move_player(ctx: &ReducerContext, x: f32, y: f32, z: f32, rotation_y: f32) {
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        player.x = x;
        player.y = y;
        player.z = z;
        player.rotation_y = rotation_y;
        player.last_seen = ctx.timestamp;
        ctx.db.player().identity().update(player);
    }
}

#[reducer]
pub fn send_chat(ctx: &ReducerContext, message: String) {
    if let Some(player) = ctx.db.player().identity().find(ctx.sender) {
        // Limit message length
        let message: String = message.chars().take(500).collect();
        if !message.trim().is_empty() {
            ctx.db.chat_message().insert(ChatMessage {
                id: 0, // Auto-incremented
                sender_identity: ctx.sender,
                sender_name: player.name.clone(),
                message,
                timestamp: ctx.timestamp,
            });
        }
    }
}

// ============================================================================
// World Generation
// ============================================================================

fn generate_world(ctx: &ReducerContext, seed: u64) {
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(seed);
    let mut counter = 0u64;

    // Generate trees
    for _ in 0..50 {
        let x = rng.gen_range(-50.0..50.0);
        let z = rng.gen_range(-50.0..50.0);

        ctx.db.world_entity().insert(WorldEntity {
            id: next_entity_id(ctx, &mut counter),
            entity_type: "tree".to_string(),
            x,
            y: 0.0,
            z,
            rotation_y: rng.gen_range(0.0..360.0),
            scale: rng.gen_range(0.8..1.2),
            metadata: serde_json::to_string(&serde_json::json!({
                "variant": rng.gen_range(0..3)
            }))
            .unwrap_or_default(),
        });
    }

    // Generate rocks
    for _ in 0..30 {
        let x = rng.gen_range(-50.0..50.0);
        let z = rng.gen_range(-50.0..50.0);

        ctx.db.world_entity().insert(WorldEntity {
            id: next_entity_id(ctx, &mut counter),
            entity_type: "rock".to_string(),
            x,
            y: 0.0,
            z,
            rotation_y: rng.gen_range(0.0..360.0),
            scale: rng.gen_range(0.5..1.5),
            metadata: "{}".to_string(),
        });
    }

    // Generate a central structure
    ctx.db.world_entity().insert(WorldEntity {
        id: next_entity_id(ctx, &mut counter),
        entity_type: "building".to_string(),
        x: 0.0,
        y: 0.0,
        z: 0.0,
        rotation_y: 0.0,
        scale: 1.0,
        metadata: serde_json::to_string(&serde_json::json!({
            "name": "Town Center",
            "style": "medieval"
        }))
        .unwrap_or_default(),
    });

    // Update counter
    ctx.db.entity_counter().insert(EntityCounter { id: 0, count: counter });
}

fn next_entity_id(ctx: &ReducerContext, counter: &mut u64) -> u64 {
    *counter += 1;
    *counter
}

// ============================================================================
// Entity Management
// ============================================================================

#[reducer]
pub fn spawn_entity(
    ctx: &ReducerContext,
    entity_type: String,
    x: f32,
    y: f32,
    z: f32,
    rotation_y: f32,
    scale: f32,
    metadata: String,
) {
    // Get next entity ID
    let mut counter = ctx.db.entity_counter().id().find(0)
        .map(|c| c.count)
        .unwrap_or(0);
    counter += 1;
    ctx.db.entity_counter().id().update(EntityCounter { id: 0, count: counter });

    ctx.db.world_entity().insert(WorldEntity {
        id: counter,
        entity_type,
        x,
        y,
        z,
        rotation_y,
        scale,
        metadata,
    });
}

#[reducer]
pub fn remove_entity(ctx: &ReducerContext, entity_id: u64) {
    ctx.db.world_entity().id().delete(entity_id);
}

#[reducer]
pub fn modify_entity(
    ctx: &ReducerContext,
    entity_id: u64,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    rotation_y: Option<f32>,
    scale: Option<f32>,
    metadata: Option<String>,
) {
    if let Some(mut entity) = ctx.db.world_entity().id().find(entity_id) {
        if let Some(v) = x { entity.x = v; }
        if let Some(v) = y { entity.y = v; }
        if let Some(v) = z { entity.z = v; }
        if let Some(v) = rotation_y { entity.rotation_y = v; }
        if let Some(v) = scale { entity.scale = v; }
        if let Some(v) = metadata { entity.metadata = v; }
        ctx.db.world_entity().id().update(entity);
    }
}

// ============================================================================
// World Regeneration
// ============================================================================

#[reducer]
pub fn regenerate_world(ctx: &ReducerContext, seed: Option<u64>) {
    // Clear existing entities
    for entity in ctx.db.world_entity().iter() {
        ctx.db.world_entity().id().delete(entity.id);
    }

    // Use provided seed or generate new one
    let seed = seed.unwrap_or_else(|| ctx.timestamp.to_micros_since_unix_epoch() as u64);

    // Update world info
    if let Some(mut info) = ctx.db.world_info().id().find(0) {
        info.seed = seed;
        ctx.db.world_info().id().update(info);
    }

    // Reset counter
    ctx.db.entity_counter().id().update(EntityCounter { id: 0, count: 0 });

    // Generate new world
    generate_world(ctx, seed);
}
