# Repository Guidelines

## Project Structure & Module Organization

MazeTrace is a Rust terminal application using `ratatui`. Source code lives in `src/`: `main.rs` handles terminal setup, `app.rs` owns the event loop state, `maze.rs` defines the model, `generator.rs` and `explorer.rs` contain algorithms, `render.rs` converts mazes to characters, `ui.rs` draws the TUI, and `config.rs` defines CLI options. Keep integration tests in `tests/`; add fixtures under `tests/fixtures/` only when reusable sample data is needed.

## Build, Test, and Development Commands

- `cargo run -- --width 21 --height 11`: run MazeTrace locally with explicit maze dimensions.
- `cargo run -- --generator prim --solver bfs --auto-start`: try alternate algorithms.
- `cargo test`: run unit and integration tests.
- `cargo fmt --all`: format Rust code before committing.
- `cargo clippy --all-targets --all-features`: catch common mistakes and style issues.
- `cargo build --release`: build an optimized binary for manual terminal testing.

## Coding Style & Naming Conventions

Use idiomatic Rust formatted by `rustfmt`. Prefer 4-space indentation, `snake_case` for functions, variables, modules, and file names, `PascalCase` for types and enum variants, and `SCREAMING_SNAKE_CASE` for constants. Keep `ratatui` drawing code in UI/rendering modules; do not mix terminal event handling, maze generation, and pathfinding in `main.rs`. Preserve Unicode maze symbols from the design doc, and provide ASCII fallback behavior where rendering code supports it.

## Testing Guidelines

Favor deterministic tests for maze generation and exploration by using fixed seeds. Unit-test pure logic in the owning module with `#[cfg(test)]`; add integration tests in `tests/*.rs` for command-level behavior. Cover wall carving, entrance/exit placement, generator variants, solver variants, final path reconstruction, and render-size calculations.

## Commit & Pull Request Guidelines

Use Conventional Commits in English, such as `feat: add prim maze generator` or `test: cover render size calculation`. Keep subjects concise and imperative.

Pull requests should include a clear summary, test results, linked issues when applicable, and terminal screenshots or short recordings for visible TUI changes. Note any limitations around terminal size, Unicode rendering, or platform support.

## Agent-Specific Instructions

Do not overwrite untracked work. Before editing, check repository status and preserve user changes. Keep contributions aligned with `docs/mazetrace-design.md` unless the user explicitly updates the product direction.
