# Repository Guidelines

## Project Structure & Module Organization

MazeTrace is planned as a Rust terminal application using `ratatui`. The current repository contains the high-level design in `docs/mazetrace-design.md`; treat it as the source of truth until the Cargo project is scaffolded. Keep future code under `src/`, with focused modules such as `maze.rs`, `generator.rs`, `explorer.rs`, `render.rs`, `ui.rs`, `input.rs`, and `config.rs`. Keep algorithm and rendering logic testable without requiring a live terminal. Put integration tests in `tests/` and reusable fixtures or sample outputs under `tests/fixtures/` if needed.

## Build, Test, and Development Commands

After `Cargo.toml` is added, use the standard Cargo workflow:

- `cargo run -- --width 21 --height 11`: run MazeTrace locally with explicit maze dimensions.
- `cargo test`: run unit and integration tests.
- `cargo fmt --all`: format Rust code before committing.
- `cargo clippy --all-targets --all-features`: catch common mistakes and style issues.
- `cargo build --release`: build an optimized binary for manual terminal testing.

If these commands are unavailable, the Rust scaffold has not been created yet.

## Coding Style & Naming Conventions

Use idiomatic Rust formatted by `rustfmt`. Prefer 4-space indentation, `snake_case` for functions, variables, modules, and file names, `PascalCase` for types and enum variants, and `SCREAMING_SNAKE_CASE` for constants. Keep `ratatui` drawing code in UI/rendering modules; do not mix terminal event handling, maze generation, and pathfinding in `main.rs`. Preserve Unicode maze symbols from the design doc, and provide ASCII fallback behavior where rendering code supports it.

## Testing Guidelines

Favor deterministic tests for maze generation and exploration by using fixed seeds. Unit-test pure logic in the owning module with `#[cfg(test)]`; add integration tests in `tests/*.rs` for command-level behavior once the binary exists. Cover wall carving, entrance/exit placement, DFS exploration, final path reconstruction, and render-size calculations.

## Commit & Pull Request Guidelines

Use Conventional Commits in English, such as `feat: add DFS maze generator` or `test: cover render size calculation`. The current history is minimal, so keep future subjects concise and imperative.

Pull requests should include a clear summary, test results, linked issues when applicable, and terminal screenshots or short recordings for visible TUI changes. Note any limitations around terminal size, Unicode rendering, or platform support.

## Agent-Specific Instructions

Do not overwrite untracked work. Before editing, check repository status and preserve user changes. Keep contributions aligned with `docs/mazetrace-design.md` unless the user explicitly updates the product direction.
