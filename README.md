# Orbital Mechanics Simulator

An interactive, terminal-based orbital mechanics calculator written in Rust.
No flags to memorize — launch it and it walks you through menus, like an
old-school engineering console rather than a typical CLI tool.

```
╔══════════════════════════════════════════════════╗
║          ORBITAL MECHANICS SIMULATOR                ║
║                       v0.5.0                        ║
║             Interactive CLI Tool                    ║
╚══════════════════════════════════════════════════╝
```

## Features

- **Orbital Mechanics** — define an orbit around Earth three different
  ways (periapsis/apoapsis, semi-major axis/eccentricity, or a position
  and velocity vector) and get back the full set of derived parameters:
  semi-major axis, eccentricity, period, velocity at periapsis/apoapsis,
  specific orbital energy, specific angular momentum, and orbit type
  (circular, elliptical, parabolic, or hyperbolic)
- **Hohmann Transfer** — the classic two-burn maneuver between two
  circular orbits: both burns' Δv, total Δv, and transfer time
- **Propulsion** — the Tsiolkovsky rocket equation, solvable in either
  direction (propellant required for a given Δv, or achievable Δv for a
  given propellant load), with proper error handling on invalid input
  instead of crashing
- **Hohmann → Propulsion integration** — after a transfer's result, jump
  straight into a propellant estimate using that transfer's Δv, with no
  re-typing numbers between menus
- High display precision throughout (6+ decimal places), since this is
  meant to work as an actual calculator — see [`DEVLOG.md`](DEVLOG.md)
  for a full breakdown of where that precision comes from and its limits

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain,
  installed via `rustup`) — no other dependencies

## Installation

```
git clone https://github.com/BakrLabs/orbital-mechanics-simulator.git
cd orbital-mechanics-simulator
cargo build --release
```

## Usage

```
cargo run
```

You'll get a title screen, then the main menu. Everything from there is
menu-driven — pick a number, answer the prompts.

```
┌──────────────────────────────────────────────┐
│                  MAIN MENU                    │
├──────────────────────────────────────────────┤
│                                                │
│  1. Orbital Mechanics                         │
│  2. Hohmann Transfer                          │
│  3. Propulsion                                │
│  4. Settings                                   │
│  5. About                                      │
│  0. Exit                                       │
│                                                │
└──────────────────────────────────────────────┘
```

### Example: a Hohmann transfer with propellant estimate

1. `2` → Hohmann Transfer → `1` → Earth
2. Initial orbit altitude: `200` km, target: `35786` km (LEO to GEO)
3. Review the transfer result (burns, total Δv, transfer time)
4. Choose `1` (Calculate required propellant), enter an engine Isp and
   spacecraft mass, and get a propellant estimate for that exact transfer

## Project layout

```
src/
├── main.rs                 entry point
├── app/                    menu logic and user-facing flows
│   ├── mod.rs
│   ├── menu.rs
│   ├── orbital_mechanics.rs
│   ├── hohmann_transfer.rs
│   └── propulsion.rs
├── physics/                orbital mechanics engine
│   ├── mod.rs
│   ├── constants.rs
│   ├── body.rs              CelestialBody (currently just Earth)
│   ├── orbit.rs              Orbit struct and its derived quantities
│   ├── orbit_type.rs
│   ├── vector2.rs
│   └── hohmann.rs            HohmannTransfer
├── propulsion/              rocket equation engine
│   ├── mod.rs
│   └── rocket_equation.rs
└── ui/                      terminal I/O
    ├── mod.rs
    ├── display.rs
    └── input.rs
```

The split follows one rule: UI → application logic → physics/engineering
engine. The menu code never does its own math, and the physics/propulsion
modules never touch stdin or stdout — they're plain functions and structs
that could be dropped into a different frontend without changes.

## Testing

```
cargo test
```

Every physics and propulsion module has unit tests, checked against
hand-derived reference values (not just the numbers a calculator run
happens to produce) — including known results like a LEO→GEO Hohmann
transfer (~3.9 km/s, the commonly cited textbook figure) and a
geostationary orbital period (~23.93 hours, one sidereal day), plus
precision checks at the extremes (near-circular orbits, very low and very
high altitudes) and full error-path coverage on the rocket equation.

## Status

Currently at `v0.5.0`, covering orbital mechanics, Hohmann transfers, and
propulsion for Earth-centered, two-dimensional, single-burn scenarios.
See [`DEVLOG.md`](DEVLOG.md) for build notes, verified corrections to a
couple of the original spec's worked examples, and ideas for where this
could go next (numerical propagation, 3D orbits, other central bodies,
visualization).

## License

MIT
