# MazeTrace

MazeTrace is an animated terminal maze generator and explorer. It builds a maze
with a step-by-step DFS backtracker, then traces a DFS route from the entrance
to the exit.

## Run

```sh
cargo run -- --width 21 --height 11
```

Useful options:

- `--speed 90`: set animation speed in steps per second.
- `--seed 1234`: reproduce the same maze.
- `--ascii`: use ASCII characters instead of Unicode line art.

## Controls

- `Space`: pause or resume
- `S`: step once
- `N`: generate a new maze
- `R`: replay exploration on the current maze
- `+` / `-`: adjust speed
- `Q` or `Esc`: quit

See [docs/mazetrace-design.md](docs/mazetrace-design.md) for the full design.
