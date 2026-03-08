---
slug: world-as-skill
title: "World as Skill: Giving Agents a Place to Live"
authors: [yi]
tags: [localgpt, gen, 3d, architecture, skills]
---

**LocalGPT Gen** introduces a new concept: treating complete 3D worlds as reusable skills that agents can save, load, and share.

<!--truncate-->

## The Problem: Worlds Are Ephemeral

When an agent builds a 3D scene, the work is often lost when the session ends. There's no easy way to:
*   Save a world and continue building later
*   Share a creation with another agent or user
*   Build a library of reusable environments

## The Solution: World Skills

With World Skills, `localgpt-gen` can save complete worlds as skill directories containing:

*   **Scene geometry** — All entities, meshes, and transforms
*   **Behaviors** — Animations like orbit, spin, bob, path following
*   **Audio configuration** — Ambient soundscapes and spatial emitters
*   **Camera tours** — Guided sequences through the world
*   **Avatar settings** — User presence and movement configuration

The skill format is simple and human-readable:

```
skills/my-world/
├── SKILL.md          # Description and usage
├── world.toml        # Manifest (environment, avatar, camera)
├── scene.glb         # glTF export of the scene
├── behaviors.toml    # Entity animations
├── audio.toml        # Sound configuration
└── tours.toml        # Guided camera tours
```

## Decoupling Brain and Body

This architecture strictly separates the **cognitive engine** (the "Brain") from the **spatial engine** (the "Body").

### The Brain: The Agent
The agent excels at reasoning, planning, and maintaining long-term memory. It is the pilot.

### The Body: LocalGPT-Gen
`localgpt-gen` is the mech. It handles the physics, rendering, asset management, and spatial state. It doesn't need to know *why* it's building a cabin; it just needs to know *how* to place the logs.

By decoupling these, we enable powerful combinations:
*   **Save and resume** — Build a world over multiple sessions
*   **Share and remix** — Export to glTF for use in other engines
*   **Template library** — Start new projects from saved worlds

## Not a Clone, But a Complement

**LocalGPT is an agent framework.** It defines how an AI thinks, remembers, and uses tools.
**LocalGPT-Gen is a world engine.** It defines a reality for that AI to inhabit.

In the spirit of the Unix philosophy, we are building small, sharp tools that communicate over standard protocols. World Skills make those tools persistent and shareable.

## Try It Out

1.  Build a world: `cargo run -p localgpt-gen`
2.  Save it as a skill: ask the agent to "save this world as a skill named my-castle"
3.  Load it later: "load the my-castle world skill"
4.  Export: the `scene.glb` file works in Blender, Unity, or any glTF viewer
