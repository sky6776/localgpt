---
slug: explorable-worlds-mcp
title: "Explorable Worlds as Agent Skills: LocalGPT-Gen as an MCP Server"
authors: [yi]
tags: [localgpt, gen, mcp, 3d, bevy, architecture]
---

We are excited to announce a major architectural shift for **LocalGPT Gen**: it now operates as a **Model Context Protocol (MCP) Server**.

This change transforms `localgpt-gen` from a standalone AI assistant into a universal "3D imagination engine" that can be plugged into *any* agent—including the entire ecosystem of "Claws" (OpenClaw, IronClaw, ZeroClaw) and major tools like Claude Desktop.

<!--truncate-->

## The Problem: Agents Live in the Void

Most AI agents today—whether it's a simple chatbot or a sophisticated autonomous system like **OpenClaw**—live in a text-based void. They can write code, query databases, and summarize documents, but they have no spatial existence. They cannot "see" a room, "place" an object, or "navigate" a terrain.

Attempts to bridge this gap usually involve heavy, complex pipelines:
*   Sending prompts to a cloud API (slow, costly, non-interactive).
*   Scripting external tools like Blender (requires 700MB+ installs, fragile Python bindings).
*   Generating raw 3D mesh data (prone to hallucinations and geometric errors).

We wanted something different: a fast, local, **explorable world** that an agent can interact with as naturally as it interacts with a file system.

## The Solution: LocalGPT-Gen as an MCP Server

With the latest update, `localgpt-gen` exposes its Bevy-based 3D engine via the **Model Context Protocol (MCP)**.

Instead of building a monolithic "World Agent," we built a **Skill Server**. When you run:

```bash
cargo run -p localgpt-gen -- mcp-server
```

You aren't starting a chatbot. You are starting a 3D environment that listens for commands. Any MCP-compliant client can connect to it and instantly gain a suite of 14 high-level spatial tools, including:

*   `gen_spawn_primitive`: Create visible 3D objects (cubes, spheres, planes) by name.
*   `gen_set_light`: Control lighting (sun, point lights, ambient) to set the mood.
*   `gen_set_camera`: Move the "eye" of the agent to frame specific views.
*   `gen_screenshot`: The most critical tool—allowing the agent to *see* what it has built.

## Why This Matters: Decoupling Brain and Body

This architecture strictly separates the **cognitive engine** (the "Brain") from the **spatial engine** (the "Body").

### The Brain: The Claw Ecosystem
The "Claw" family of agents—**IronClaw** (TUI), **OpenClaw** (Core), **ZeroClaw** (Hardware), and **PicoClaw** (Embedded)—excel at reasoning, planning, and maintaining long-term memory. They are the pilots.

### The Body: LocalGPT-Gen
`localgpt-gen` is the mech. It handles the physics, rendering, asset management, and spatial state. It doesn't need to know *why* it's building a cabin; it just needs to know *how* to place the logs.

By decoupling these, we enable powerful combinations:
*   **IronClaw + Gen:** A terminal-based agent that pops up a 3D window to visualize the infrastructure it's managing.
*   **ZeroClaw + Gen:** A Raspberry Pi robot that "imagines" a path before physically moving, using the Gen engine as a simulator.
*   **Claude Desktop + Gen:** A drag-and-drop workflow where you ask Claude to "mock up a stage design" and it appears instantly in a local window.

## Not a Clone, But a Complement

There has been some confusion about whether `localgpt-gen` is "just another OpenClaw." It is not.

**OpenClaw is an agent framework.** It defines how an AI thinks, remembers, and uses tools.
**LocalGPT-Gen is a world engine.** It defines a reality for that AI to inhabit.

We are not competing with the Claws; we are giving them a place to live. In the spirit of the Unix philosophy, we are building small, sharp tools that communicate over standard protocols. `localgpt-gen` is the standard tool for "local, 3D spatial reasoning."

## Technical Highlights

*   **Single Binary:** No external dependencies. No Blender. No Python. Just a Rust binary powered by the [Bevy Game Engine](https://bevyengine.org/).
*   **Intent-Level API:** The MCP tools are designed for LLMs. Agents don't say "Create Entity ID 504 with Mesh Handle 12"; they say "Spawn a red chair at [2, 0, 2]."
*   **Visual Feedback Loop:** The `gen_screenshot` tool closes the loop, allowing the agent to self-correct ("The roof is crooked, let me rotate it 5 degrees").

## Try It Out

The new MCP server mode is available in the latest build.

1.  Clone the repo.
2.  Run the server: `cargo run -p localgpt-gen -- mcp-server`
3.  Configure your agent (Claude Desktop, OpenClaw, etc.) to connect to the stdio stream.
4.  Prompt: *"Build a modern house with a flat roof and a swimming pool."*

Welcome to the world, agents.
