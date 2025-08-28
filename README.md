# 🍅 mato

![demo](./assets/demo.gif)

## Features

- Customizable timers
- Desktop notifications
- Cycle counts and limit

## Installation (via `cargo`)

```nushell
cargo install mato
```

## Usage

```
Pomodoro TUI timer

Usage: mato [OPTIONS]

Options:
  -t, --tick <TICK>               TUI render interval in milliseconds [default: 1000]
  -n, --names <NAMES>...          Name for each timer [default: Work Break]
  -d, --durations <DURATIONS>...  Duration for each timer, e.g. 3h2m1s [default: 25m 5m]
  -c, --cycles <CYCLES>           Limit number of cycles, 0 to set no limits [default: 0]
  -N                              Disable notifications
  -h, --help                      Print help
  -V, --version                   Print version
```
