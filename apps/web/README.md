# LocalGPT World - SpacetimeDB Multiplayer

A multiplayer 3D world powered by SpacetimeDB and React Three Fiber.

## Structure

```
apps/web/
├── client/           # React + Vite + Three.js frontend
│   └── src/
│       ├── components/   # React components
│       ├── hooks/        # SpacetimeDB hooks
│       └── pages/        # Route pages
├── server/           # SpacetimeDB Rust module
│   └── src/lib.rs    # Tables, reducers, world gen
└── shared/           # Generated TypeScript bindings
```

## Quick Start

### Prerequisites

- [SpacetimeDB CLI](https://github.com/clockworklabs/SpacetimeDB) installed
- Node.js 18+ and npm
- Rust 1.70+

### Start SpacetimeDB

```bash
# Start local SpacetimeDB instance
spacetime start

# In another terminal, publish the module
cd apps/web/server
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

- **player** - Connected players with position
- **world_entity** - Generated world objects (trees, rocks, buildings)
- **chat_message** - Chat history
- **world_info** - World metadata (seed, size)

### Reducers

- `move_player(x, y, z, rotation)` - Update player position
- `send_chat(message)` - Send chat message
- `spawn_entity(...)` - Add new world entity
- `regenerate_world(seed)` - Regenerate world

### Client Stack

- **React** - UI framework
- **Three.js / React Three Fiber** - 3D rendering
- **Zustand** - State management
- **SpacetimeDB SDK** - Real-time sync
