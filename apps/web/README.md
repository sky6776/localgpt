# LocalGPT World - Web Client

A multiplayer 3D world web client using React Three Fiber. Connects to the SpacetimeDB server in `crates/spacetime/`.

## Structure

```
apps/web/
├── client/           # React + Vite + Three.js frontend
│   └── src/
│       ├── components/   # React components
│       ├── hooks/        # SpacetimeDB hooks
│       └── pages/        # Route pages

crates/spacetime/     # SpacetimeDB Rust module (backend)
```

## Shared Types with Gen Crate

The server reuses types from `crates/world-types`:

- `EntityId`, `EntityName`, `EntityRef` — Identity types
- `WorldEntity`, `WorldTransform` — Entity structure
- `Shape` — Parametric shapes (Cuboid, Sphere, etc.)
- `MaterialDef`, `LightDef`, `BehaviorDef`, `AudioDef` — Components
- `ChunkCoord` — Spatial partitioning (64×64 chunks)

## Quick Start

### Prerequisites

- [SpacetimeDB CLI](https://github.com/clockworklabs/SpacetimeDB) 2.0+
- Node.js 18+ and npm
- Rust 1.70+

### Start SpacetimeDB

```bash
# Start local SpacetimeDB instance
spacetime start

# In another terminal, publish the module
cd crates/spacetime
spacetime publish --server local localgpt-world
```

### Start the Client

```bash
cd apps/web/client
npm install
npm run dev
```

Open http://localhost:3000

## Development

### Generate TypeScript Bindings

After modifying the Rust module:

```bash
cd apps/web/client
npm run generate
```

### Build for Production

```bash
cd apps/web/client
npm run build
```

## Architecture

### SpacetimeDB Tables

| Table | Purpose |
|-------|---------|
| `player` | Connected players with position, device type |
| `world_entity` | World objects with serialized components |
| `chat_message` | Chat and system messages |
| `world_info` | World metadata (seed, size, timestamps) |
| `chunk_subscription` | Mobile streaming subscriptions |
| `entity_counter` | Auto-increment ID counter |

### Client Stack

- **React 18** — UI framework
- **Three.js / React Three Fiber** — 3D rendering
- **@clockworklabs/spacetimedb-sdk 2.0** — Real-time sync
- **Zustand** — State management

### Mobile Support

The SpacetimeDB server in `crates/spacetime/` supports mobile clients via:
- Device type tracking (`web`, `ios`, `android`)
- Chunk-based streaming for bandwidth efficiency
- Lightweight position sync protocol

## Integration with Gen Crate

The gen crate (`crates/gen`) can sync worlds to SpacetimeDB:

1. Gen creates `WorldEntity` instances
2. Serialize to JSON: `serde_json::to_string(&entity)`
3. Call `spawn_world_entity` reducer
4. Changes sync to all connected clients

## Environment Variables

```bash
VITE_SPACETIMEDB_URL=ws://localhost:3000
VITE_MODULE_NAME=localgpt-world
```
