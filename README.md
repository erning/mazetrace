# MazeTrace

MazeTrace is an animated terminal maze generator and explorer. It builds a maze
step by step, waits at the completed maze, then traces a solver route from the
entrance to the exit.

## Run

```sh
cargo run -- --width 21 --height 11 --generator wilson --solver dijkstra
```

Useful options:

- `--generator dfs|prim|kruskal|aldous-broder|wilson|recursive-division`:
  choose the maze generation algorithm.
- `--solver dfs|bfs|astar|dijkstra|dead-end|wall-follower`:
  choose the solving algorithm.
- `--auto-start`: start solving immediately after generation completes.
- `--speed 90`: set animation speed in steps per second.
- `--seed 1234`: reproduce the same maze.
- `--ascii`: use ASCII characters instead of Unicode line art.

## Controls

- `Space`: start exploration from Ready, or pause/resume animation
- `S`: step once
- `N`: generate a new maze
- `R`: reset exploration on the current maze
- `+` / `-`: adjust speed
- `1` to `6`: switch solver to DFS, BFS, A*, Dijkstra, Dead-End, or
  Wall-Follower
- `Q` or `Esc`: quit

See [docs/mazetrace-design.md](docs/mazetrace-design.md) for the full design.
