---
sidebar_position: 14.5
---

# MCP Server

LocalGPT Gen can run as an [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) server, exposing gen tools and core LocalGPT tools over stdio. This lets external AI coding assistants — Claude CLI, Gemini CLI, Codex CLI, and MCP-compatible editors like VS Code and Zed — drive the Bevy 3D window directly, with full access to LocalGPT's memory system.

## Why MCP?

When using LocalGPT Gen in its default interactive mode, the built-in LLM agent calls gen tools directly inside the same process. But if you want to use a different AI backend — Claude CLI, Gemini CLI, Codex, or an editor's built-in AI — those tools aren't accessible because they run in separate processes.

MCP solves this. It's a standard protocol that these tools already support. By running `localgpt-gen --mcp-server`, the Bevy window opens and all gen tools become available to any MCP client over stdio. The AI backend becomes the orchestrator — it manages the conversation, calls tools, and drives the scene building.

## Quick Start

```bash
# Start Gen as an MCP server (Bevy window opens, tools served over stdio)
localgpt-gen --mcp-server
```

Then configure your AI tool to connect to it (see sections below).

## Available Tools

The MCP server exposes gen tools plus core LocalGPT tools:

### Gen Tools (28 tools)

- **Scene query** — `gen_scene_info`, `gen_screenshot`, `gen_entity_info`
- **Entity creation** — `gen_spawn_primitive`, `gen_spawn_entities`, `gen_spawn_mesh`, `gen_load_gltf`
- **Entity modification** — `gen_modify_entity`, `gen_modify_entities`, `gen_delete_entity`, `gen_delete_entities`
- **Camera & environment** — `gen_set_camera`, `gen_set_light`, `gen_set_environment`
- **Audio** — `gen_set_ambience`, `gen_audio_emitter`, `gen_modify_audio`, `gen_audio_info`
- **Behaviors** — `gen_add_behavior`, `gen_remove_behavior`, `gen_list_behaviors`, `gen_pause_behaviors`
- **World skills** — `gen_save_world`, `gen_load_world`, `gen_export_world`, `gen_clear_scene`
- **Export** — `gen_export_screenshot`, `gen_export_gltf`
- **Undo/Redo** — `gen_undo`, `gen_redo`, `gen_undo_info`

See [Gen Tools Reference](/docs/gen/tools) for full documentation on each tool.

### Core Tools

- **`memory_search`** — Search LocalGPT's memory (MEMORY.md, daily logs) using hybrid semantic + keyword search. The AI backend can recall preferences, past decisions, and context from previous sessions.
- **`memory_get`** — Fetch specific lines from memory files. Use after `memory_search` to read relevant snippets.
- **`web_fetch`** — Fetch and extract content from URLs. Useful for referencing documentation or inspiration during scene building.
- **`web_search`** — Search the web (if configured in `config.toml`). Useful for researching scene concepts.

### Why Not File/Shell Tools?

CLI tools like `bash`, `read_file`, `write_file`, and `edit_file` are **not** exposed via MCP. External AI backends (Claude CLI, Gemini CLI, Codex) already have their own file and shell tools. Exposing duplicates would create confusion and security concerns.

## Claude CLI

Add to `~/.claude.json` (or project-level `.claude/settings.json`):

```json
{
  "mcpServers": {
    "localgpt-gen": {
      "command": "localgpt-gen",
      "args": ["--mcp-server"]
    }
  }
}
```

Then start Claude CLI as usual. The gen tools appear alongside Claude's built-in tools. Ask it to build a scene:

```
$ claude
> Build a medieval castle with a moat, drawbridge, and warm torchlight
```

Claude will call `gen_spawn_primitive`, `gen_set_light`, `gen_set_camera`, etc. to construct the scene in the Bevy window.

## Gemini CLI

Add to `~/.gemini/settings.json`:

```json
{
  "mcpServers": {
    "localgpt-gen": {
      "command": "localgpt-gen",
      "args": ["--mcp-server"]
    }
  }
}
```

## OpenAI Codex CLI

Add to `~/.codex/config.json`:

```json
{
  "mcpServers": {
    "localgpt-gen": {
      "command": "localgpt-gen",
      "args": ["--mcp-server"]
    }
  }
}
```

## VS Code (Copilot)

VS Code supports MCP servers through its Copilot agent mode. Add to your workspace `.vscode/settings.json` or user settings:

```json
{
  "mcp": {
    "servers": {
      "localgpt-gen": {
        "command": "localgpt-gen",
        "args": ["--mcp-server"]
      }
    }
  }
}
```

You can also add it via the command palette: **MCP: Add Server** and choose "stdio" transport.

Once configured, use Copilot in agent mode (`@workspace`) and ask it to build 3D scenes. The gen tools show up as available tools that Copilot can call.

## Zed Editor

Add to your Zed settings (`~/.config/zed/settings.json`):

```json
{
  "context_servers": {
    "localgpt-gen": {
      "command": {
        "path": "localgpt-gen",
        "args": ["--mcp-server"]
      }
    }
  }
}
```

The gen tools become available in Zed's AI assistant panel.

## Cursor

Add to your Cursor MCP configuration (`.cursor/mcp.json` in your project or global config):

```json
{
  "mcpServers": {
    "localgpt-gen": {
      "command": "localgpt-gen",
      "args": ["--mcp-server"]
    }
  }
}
```

## Windsurf

Add to your Windsurf MCP configuration (`~/.codeium/windsurf/mcp_config.json`):

```json
{
  "mcpServers": {
    "localgpt-gen": {
      "command": "localgpt-gen",
      "args": ["--mcp-server"]
    }
  }
}
```

## How It Works

```
┌─────────────────────┐       MCP stdio (JSON-RPC)        ┌──────────────────┐
│  AI Backend         │◄──────────────────────────────────►│  localgpt-gen    │
│  (Claude, Gemini,   │        tools/list                  │                  │
│   Codex, VS Code,   │        tools/call                  │  MCP Server      │
│   Zed, Cursor)      │                                    │    │       │     │
└─────────────────────┘                                    │ GenBridge Memory │
        ▲                                                  │    ↓       ↓     │
        │ manages conversation,                            │  Bevy   SQLite   │
        │ decides which tools                              │  3D     FTS5 +   │
        │ to call and when                                 │ Engine  vectors  │
        │                                                  └──────────────────┘
   AI Backend is the
   orchestrator
```

In MCP mode, the **AI backend is the orchestrator**. It manages the conversation, decides which tools to call, and drives scene building. LocalGPT Gen provides the runtime (Bevy 3D engine + memory database) and exposes it through standard MCP tools.

1. The AI backend spawns `localgpt-gen --mcp-server` as a child process
2. MCP handshake happens over stdio (JSON-RPC 2.0, one message per line)
3. The backend discovers all tools via `tools/list` (gen tools + memory + web)
4. The AI reasons about the scene and calls tools as needed — `gen_spawn_primitive`, `memory_search`, `gen_screenshot`, etc.
5. Gen tool calls are dispatched through the GenBridge channel to the Bevy main thread; memory tool calls query the LocalGPT SQLite database
6. Results are sent back to the AI backend, which continues building

This is different from LocalGPT Gen's interactive mode, where LocalGPT's own agent loop is the orchestrator. In MCP mode, LocalGPT doesn't run its agent loop at all — it's purely a tool server.

## Combining with Scene File

You can load an existing scene while starting the MCP server:

```bash
localgpt-gen --mcp-server --scene ./my-scene.glb
```

The AI backend can then modify the pre-loaded scene.

## Memory Integration

The MCP server initializes LocalGPT's memory system using the workspace configured in `~/.localgpt/config.toml`. This means:

- **`memory_search`** queries the same MEMORY.md, daily logs, and knowledge files used by LocalGPT's interactive mode
- If embeddings are enabled (`memory.embedding_provider = "local"`), semantic search works across all indexed memory chunks
- The AI backend can tell the user about past sessions, preferences, and decisions stored in memory

To write to memory, the AI backend can use its own file tools (e.g., Claude CLI's `Write` tool) to edit `MEMORY.md` or `memory/*.md` files in the workspace directory. The workspace path is shown in verbose mode.

## Tips

- **Verbose logging**: Add `--verbose` to see MCP protocol messages and tool list in stderr: `localgpt-gen --mcp-server --verbose`
- **Binary path**: If `localgpt-gen` is not in your `$PATH`, use the full path (e.g., `/Users/you/.cargo/bin/localgpt-gen`) in the MCP server configuration
- **One instance**: Each MCP server config spawns its own Bevy window. Only one instance should run at a time per display
- **Screenshots**: The AI can take screenshots via `gen_screenshot` to see what it built and course-correct — this works the same as in interactive mode
- **Memory workspace**: The MCP server reads memory from the same workspace as `localgpt chat`. Any notes saved in interactive mode are available via `memory_search` in MCP mode
