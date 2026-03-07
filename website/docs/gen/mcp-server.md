---
sidebar_position: 14.5
---

# MCP Server

LocalGPT Gen can run as an [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) server, exposing all gen tools over stdio. This lets external AI coding assistants — Claude CLI, Gemini CLI, Codex CLI, and MCP-compatible editors like VS Code and Zed — drive the Bevy 3D window directly.

## Why MCP?

When using LocalGPT Gen in its default interactive mode, the built-in LLM agent calls gen tools directly inside the same process. But if you want to use a different AI backend — Claude CLI, Gemini CLI, Codex, or an editor's built-in AI — those tools aren't accessible because they run in separate processes.

MCP solves this. It's a standard protocol that these tools already support. By running `localgpt-gen --mcp-server`, the Bevy window opens and all gen tools become available to any MCP client over stdio.

## Quick Start

```bash
# Start Gen as an MCP server (Bevy window opens, tools served over stdio)
localgpt-gen --mcp-server
```

Then configure your AI tool to connect to it (see sections below).

## Available Tools

All 31 gen tools are exposed through MCP — the same tools available in interactive mode:

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
│   Zed, Cursor)      │                                    │    ↓             │
└─────────────────────┘                                    │  GenBridge       │
                                                           │    ↓ (channels)  │
                                                           │  Bevy 3D Engine  │
                                                           └──────────────────┘
```

1. The AI backend spawns `localgpt-gen --mcp-server` as a child process
2. MCP handshake happens over stdio (JSON-RPC 2.0, one message per line)
3. The backend discovers all gen tools via `tools/list`
4. When the AI decides to call a tool, it sends `tools/call` with the tool name and JSON arguments
5. The MCP server dispatches the command through the GenBridge channel to the Bevy main thread
6. Bevy executes the command (spawn entity, set camera, etc.) and returns the result
7. The MCP server sends the result back to the AI backend

## Combining with Scene File

You can load an existing scene while starting the MCP server:

```bash
localgpt-gen --mcp-server --scene ./my-scene.glb
```

The AI backend can then modify the pre-loaded scene.

## Tips

- **Verbose logging**: Add `--verbose` to see MCP protocol messages in stderr: `localgpt-gen --mcp-server --verbose`
- **Binary path**: If `localgpt-gen` is not in your `$PATH`, use the full path (e.g., `/Users/you/.cargo/bin/localgpt-gen`) in the MCP server configuration
- **One instance**: Each MCP server config spawns its own Bevy window. Only one instance should run at a time per display
- **Screenshots**: The AI can take screenshots via `gen_screenshot` to see what it built and course-correct — this works the same as in interactive mode
