# Vis-Viva

[![CI](https://github.com/BakrLabs/vis-viva-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/BakrLabs/vis-viva-cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust edition](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](Cargo.toml)
[![Dependencies](https://img.shields.io/badge/dependencies-none-brightgreen.svg)](Cargo.toml)

An interactive, terminal-based orbital mechanics calculator written in Rust.
No flags to memorize — launch it and it walks you through menus, like an
old-school engineering console rather than a typical CLI tool. Named after
the vis-viva equation, the core formula this tool uses to compute orbital
velocity, which sits at the heart of nearly every calculation it does.

```
╔══════════════════════════════════════════════════╗
║                     VIS-VIVA                     ║
║           Orbital Mechanics Simulator            ║
║                      v1.0.0                      ║
║                                                   ║
║               Interactive CLI Tool               ║
╚══════════════════════════════════════════════════╝
```

## Features

- **Four central bodies** — Earth, Moon, Mars, and the Sun, each with its
  own real gravitational parameter and radius. Every menu that needs a
  central body (Orbital Mechanics, Hohmann Transfer, Orbit Propagation)
  offers all four
- **Orbital Mechanics** — define an orbit three different ways
  (periapsis/apoapsis, semi-major axis/eccentricity, or a position and
  velocity vector) and get back the full set of derived parameters:
  semi-major axis, eccentricity, period, velocity at periapsis/apoapsis,
  specific orbital energy, specific angular momentum, and orbit type
  (circular, elliptical, parabolic, or hyperbolic)
- **3D Orbital Elements** — the same orbit-definition idea, but in three
  dimensions: enter the six classical (Keplerian) elements directly
  (semi-major axis, eccentricity, inclination, RAAN, argument of
  periapsis, true anomaly), or a 3D position/velocity vector, and get
  back the full element set either way
- **Hohmann Transfer** — the classic two-burn maneuver between two
  circular orbits: both burns' Δv, total Δv, and transfer time
- **Propulsion** — the Tsiolkovsky rocket equation, solvable in either
  direction (propellant required for a given Δv, or achievable Δv for a
  given propellant load), with proper error handling on invalid input
  instead of crashing
- **Orbit Propagation** — numerically steps a spacecraft's position and
  velocity forward through time using 4th-order Runge-Kutta (RK4)
  integration, in both 2D and 3D, with a fixed-step mode and an
  adaptive-step mode. Shows the final state alongside a
  conservation-of-energy and conservation-of-angular-momentum table, so
  numerical accuracy is something you can see, not just something claimed
- **Hohmann → Propulsion integration** — after a transfer's result, jump
  straight into a propellant estimate using that transfer's Δv, with no
  re-typing numbers between menus
- **3D Orbital Elements → Propagation integration** — after defining a
  3D orbit, propagate it for one full period with a single choice,
  instead of writing down the state vector and re-entering it elsewhere
- High display precision throughout (6+ decimal places), since this is
  meant to work as an actual calculator — see [`DEVLOG.md`](DEVLOG.md)
  for a full breakdown of where that precision comes from and its limits

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain,
  installed via `rustup`) — no other dependencies

## Installation

```
git clone https://github.com/BakrLabs/vis-viva-cli.git
cd vis-viva-cli
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
│  4. Orbit Propagation                          │
│  5. Settings                                   │
│  6. About                                      │
│  0. Exit                                       │
│                                                │
└──────────────────────────────────────────────┘
```

### Example: a Hohmann transfer with propellant estimate

1. `2` → Hohmann Transfer → `1` (Earth)
2. Initial orbit altitude: `200` km, target: `35786` km (LEO to GEO)
3. Review the transfer result (burns, total Δv, transfer time)
4. Choose `1` (Calculate required propellant), enter an engine Isp and
   spacecraft mass, and get a propellant estimate for that exact transfer

### Example: propagating a circular orbit for one full period

1. `4` → Orbit Propagation → `1` (2D, fixed step) → `1` (Earth)
2. Position: `x = 6878.137`, `y = 0` (km) — a 500km altitude circular orbit
3. Velocity: `vx = 0`, `vy = 7.6126` (km/s) — circular speed at that radius
4. Duration: `5676` (seconds, about one orbital period), step: `1` (second)
5. The final position/velocity should land back almost exactly where it
   started, and the conservation table should show energy and angular
   momentum barely moving across the whole run

### Example: a 3D orbit with real inclination, propagated in one go

1. `1` → Orbital Mechanics → `1` (Earth) → `4` (3D Orbital Elements)
2. Semi-major axis: `7000` km, eccentricity: `0.1`
3. Inclination: `28.5` deg (a typical Cape Canaveral-launched orbit),
   RAAN: `45` deg, argument of periapsis: `90` deg, true anomaly: `120` deg
4. The result shows all six elements back, plus period — then choose
   `1` (Propagate this orbit for one period) to see it stepped through
   time via adaptive RK4, with no need to re-enter the state vector

### Example: adaptive vs. fixed step size on a highly eccentric orbit

1. `4` → Orbit Propagation → `3` (3D, adaptive step) → `1` (Earth)
2. Position: `x = 6578.137`, `y = 0`, `z = 0` (km), velocity:
   `vx = 0`, `vy = 10.972805371`, `vz = 0` (km/s) — a 200km perigee,
   e≈0.987 orbit, the same one used as a stress test in the DEVLOG
3. Duration: `3587170.629` (one full period, ~41.5 days), relative
   tolerance: `1e-9`, initial step guess: `60` (seconds)
4. Compare the step count shown at the end against a fixed-step run of
   the same orbit (menu 2) at a small step size — adaptive gets
   comparable accuracy in a small fraction of the steps

### Example: a 3D propagation around the Moon

1. `4` → Orbit Propagation → `2` (3D) → `2` (Moon)
2. Position: `x = 1838.09`, `y = 0`, `z = 0` (km) — 100km altitude above
   the Moon's surface
3. Velocity: `vx = 0`, `vy = 1.41439`, `vz = 0.8166` (km/s) — circular
   speed at that altitude, split across y/z for a 30° inclined orbit
4. Pick a duration and step size, and watch the conservation table hold
   steady even with inclination in the mix

## Project layout

```
src/
├── main.rs                 entry point
├── app/                    menu logic and user-facing flows
│   ├── mod.rs
│   ├── menu.rs
│   ├── central_body.rs       shared 4-body selection menu
│   ├── orbital_mechanics.rs
│   ├── orbital_elements_3d.rs
│   ├── hohmann_transfer.rs
│   ├── propulsion.rs
│   └── propagation.rs        both 2D and 3D propagation flows
├── physics/                orbital mechanics engine
│   ├── mod.rs
│   ├── constants.rs
│   ├── body.rs               CelestialBody: Earth, Moon, Mars, Sun
│   ├── orbit.rs               Orbit struct and its derived quantities
│   ├── orbit_type.rs
│   ├── orbital_elements.rs    3D classical orbital elements
│   ├── vector2.rs
│   ├── vector3.rs
│   ├── hohmann.rs             HohmannTransfer
│   ├── propagator.rs          RK4 numerical propagation (2D, fixed step)
│   └── propagator_3d.rs       RK4 numerical propagation (3D, fixed + adaptive step)
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

3D orbital elements and 3D propagation live alongside the original 2D
code rather than replacing it — see [`DEVLOG.md`](DEVLOG.md) for why.

## Testing

```
cargo test
```

Every push and pull request also runs `cargo fmt --check`, a full build,
and the test suite via GitHub Actions (see the CI badge above and
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)) — the badge only
turns green once the workflow has actually run, so it'll show as pending
until the first push to a real repo.

Every physics and propulsion module has unit tests, checked against
hand-derived reference values (not just the numbers a calculator run
happens to produce) — including known results like a LEO→GEO Hohmann
transfer (~3.9 km/s, the commonly cited textbook figure), a geostationary
orbital period (~23.93 hours, one sidereal day), RK4 propagation tests
(2D and 3D, fixed and adaptive step) that verify a circular or eccentric
orbit returns to its starting state after exactly one closed-form
period, a state-vector ↔ orbital-elements round trip verified to
floating-point precision, Moon/Mars/Sun surface gravity checked against
commonly cited values, cross-checks that the 3D and 2D code paths
agree wherever they overlap (equatorial orbits, planar propagation), and
a check that the adaptive propagator's relative tolerance behaves
consistently across wildly different orbit scales (Moon vs. Sun) —
plus precision checks at the extremes and full error-path coverage on
the rocket equation. The adaptive propagator's tests also pin its actual
behavior at the floating-point noise floor (see DEVLOG) rather than an
idealized version of it.

## Status

`v1.0.0` - the first stable release. This is a hardening version, not a
new-feature version: everything from v0.1 through v0.9 (orbital
mechanics in 2D and 3D, Hohmann transfers, propulsion, RK4 propagation
with fixed and adaptive step size, four central bodies) got a
consistency pass rather than a new capability. The adaptive propagator's
tolerance was switched from absolute position error (meters) to
relative error, matching the convention real orbit propagation tools
(GMAT, STK) use - verified to behave consistently across the largest
scale gap in the app (a close orbit around the Moon vs. one around the
Sun, roughly a 400x difference in radius). Mission design tools and
visualization remain explicitly out of scope for 1.0, not half-built.
See [`DEVLOG.md`](DEVLOG.md) for the full verification history,
including corrections to a couple of the original spec's worked
examples, an eccentric-orbit stress test of the fixed-step propagator,
and the honest limitations of the adaptive step-size algorithm.
## License

MIT
