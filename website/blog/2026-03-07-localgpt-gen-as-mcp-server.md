---
slug: localgpt-gen-as-mcp-server
title: "LocalGPT Gen as an MCP Server"
authors: [yi]
tags: [localgpt, gen, mcp, 3d, architecture]
draft: true
---

**LocalGPT Gen** now operates as a **Model Context Protocol (MCP) Server**, transforming it from a standalone AI assistant into a universal "3D imagination engine" that can be plugged into any MCP-compliant client.

<!--truncate-->

## The Problem: Agents Live in the Void

Most AI agents today—whether it's a simple chatbot or a sophisticated autonomous system—live in a text-based void. They can write code, query databases, and summarize documents, but they have no spatial existence. They cannot "see" a room, "place" an object, or "navigate" a terrain.

Attempts to bridge this gap usually involve heavy, complex pipelines:
*   Sending prompts to a cloud API (slow, costly, non-interactive).
*   Scripting external tools like Blender (requires 700MB+ installs, fragile Python bindings).
*   Generating raw 3D mesh data (prone to hallucinations and geometric errors).

We wanted something different: a fast, local, **explorable world** that an agent can interact with as naturally as it interacts with a file system.

## The Solution: LocalGPT-Gen as an MCP Server

`localgpt-gen` exposes its 3D engine via the **Model Context Protocol (MCP)**.

Instead of building a monolithic "World Agent," we built a **Skill Server**. When you run:

```bash
cargo run -p localgpt-gen -- mcp-server
```

You aren't starting a chatbot. You are starting a 3D environment that listens for commands. Any MCP-compliant client can connect to it and instantly gain a suite of high-level spatial tools, including:

*   `gen_spawn_primitive`: Create visible 3D objects (cubes, spheres, planes) by name.
*   `gen_set_light`: Control lighting (sun, point lights, ambient) to set the mood.
*   `gen_set_camera`: Move the "eye" of the agent to frame specific views.
*   `gen_screenshot`: The most critical tool—allowing the agent to *see* what it has built.

## Technical Highlights

*   **Single Binary:** No external dependencies. No Blender. No Python. Just a Rust binary.
*   **Intent-Level API:** The MCP tools are designed for LLMs. Agents don't say "Create Entity ID 504 with Mesh Handle 12"; they say "Spawn a red chair at [2, 0, 2]."
*   **Visual Feedback Loop:** The `gen_screenshot` tool closes the loop, allowing the agent to self-correct ("The roof is crooked, let me rotate it 5 degrees").

## Supported Clients

The MCP server integrates with:
*   **Claude CLI** — Anthropic's official CLI tool
*   **Gemini CLI** — Google's Gemini CLI
*   **Codex CLI** — OpenAI's coding agent
*   **Claude Desktop** — Desktop application with MCP support
*   **VS Code / Zed** — MCP-compatible editors

## Try It Out

1.  Clone the repo.
2.  Run the server: `cargo run -p localgpt-gen -- mcp-server`
3.  Configure your agent to connect to the stdio stream.
4.  Prompt: *"Build a modern house with a flat roof and a swimming pool."*

Welcome to the world, agents.
