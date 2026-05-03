# tiny-agentskills-framework

A small Rust proof of concept for experimenting with agent skills in a terminal chat workflow.

The app runs a Ratatui chat interface, streams assistant responses from an OpenAI-compatible chat API, exposes local skills to the agent, supports local tool calls, and asks for approval before executing tools.

## Requirements

- Rust toolchain: `1.92` or newer
- Cargo
- An OpenRouter/OpenAI-compatible API key

The crate uses Rust edition `2024`.

## Configuration

Create a `.env` file in the project root:

```env
OPENROUTER_API_KEY=your_api_key_here
OPENROUTER_MODEL=your_model_name_here
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
```

`OPENROUTER_BASE_URL` is optional. If it is not set, the app defaults to `https://openrouter.ai/api/v1`.

## Skills

Skills live under the `.skills` directory. Each skill should be placed in its own folder with a `SKILL.md` file:

```text
.skills/
  my-skill/
    SKILL.md
    references/
      extra_context.md
```

`SKILL.md` should include YAML front matter with at least a `name` and `description`:

```md
---
name: my-skill
description: When and how the agent should use this skill.
---

# Skill instructions

Write the instructions the agent should follow for this skill.
```

At startup, the app reads `.skills/*/SKILL.md`, adds the available skills to the system prompt, and lets the agent read skill files or references through the `ReadSkill` tool.

## Build

```sh
cargo build
```

For a release build:

```sh
cargo build --release
```

## Run

```sh
cargo run
```

Inside the UI:

- Type a message and press `Enter` to send it.
- Press `Ctrl+X` to exit.
- Use `PageUp`, `PageDown`, or the mouse wheel to scroll chat history.
- When a tool call is requested, approve or deny it from the overlay prompt.
