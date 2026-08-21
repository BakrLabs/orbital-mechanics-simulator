# Dev Log

Notes from building this version by version. Not required reading to use
the tool - this is here for anyone curious about the "why" behind some of
the decisions, or checking my work on the physics.

## On the spec's worked examples

This project was built from a written spec with worked numerical
examples included, meant to double as expected test output. Three of
those examples didn't hold up when checked by hand:

**Periapsis/apoapsis velocities (v0.2).** For a 200km × 400km altitude
orbit around Earth, the spec's example gave periapsis/apoapsis velocity
as 7.784/7.669 km/s. Running the vis-viva equation - the same equation
the spec itself specifies - on that same orbit gives 7.842/7.611 km/s
instead. I checked this against angular momentum conservation (velocity
times radius should be equal at both periapsis and apoapsis, since
angular momentum doesn't change over the orbit) - it holds for 7.842/
7.611 and does not hold for 7.784/7.669. Semi-major axis, eccentricity,
period, and energy in that same example were all correct as given.

**Hohmann transfer Δv (v0.4).** Same 200km→400km example. The spec's
individual burn figures (0.117 km/s and 0.114 km/s, 0.231 km/s total)
come out to almost exactly double what the stated vis-viva formulas
actually produce for that transfer (0.058 km/s and 0.058 km/s, 0.116
km/s total). The circular velocities, transfer semi-major axis, and
transfer time in the same example were all correct. As a second check
independent of the example entirely: total Δv for a Hohmann transfer
should be identical whether you're raising an orbit or lowering it
between the same two altitudes, by the symmetry of the maneuver - my
numbers satisfy that property, the spec's original figures don't.

**Rocket equation (v0.5).** No discrepancy here - the spec's worked
example (Δv = 9.40 km/s, Isp = 450s, m0 = 10000kg → final mass ≈
1188.285 kg) checked out cleanly against hand calculation.

In all three cases with a discrepancy, I went with what the stated
equations actually produce rather than the example's output, and added a
test that would fail if the calculation ever drifted toward the
incorrect example values instead.

## On numerical accuracy

Since this is meant to work as an actual calculator, it's worth being
specific about where the accuracy comes from and where its limits are.

**The equations are exact, not approximated.** Vis-viva, Kepler's third
law, the Hohmann transfer equations, and the Tsiolkovsky rocket equation
are all closed-form - there's no numerical integration, no iterative
solver, no truncated series anywhere in this project as it stands.
Given the equations are correct (the LEO→GEO Hohmann transfer test
matches the commonly cited ~3.9 km/s figure, and GEO period comes out to
~23.93 hours, a sidereal day, as it should), the only source of error
left is floating-point representation itself.

**Everything runs in `f64`** (64-bit double precision), which carries
about 15-16 significant decimal digits. For realistic orbital altitudes
and spacecraft masses, that's far more precision than the input
measurements going into these calculations could ever justify - the
arithmetic isn't the limiting factor.

**Cancellation error was checked for specifically.** Vis-viva
(`v = sqrt(mu * (2/r - 1/a))`) subtracts two numbers that get close to
each other for near-circular orbits, which is a classic way to quietly
lose precision. Comparing the direct formula against an algebraically
equivalent rearranged version, down to orbits circular to within 1
meter, they agree to the full precision of `f64` with no measurable
drift.

**Where this would eventually stop being enough:** long propagation
times (years, not single transfers) would let floating-point error
accumulate across many chained operations, which is the kind of thing
that calls for compensated summation or a higher-precision numeric type.
Not a concern for the single-orbit, single-transfer, single-burn
calculations this project does today, but worth flagging honestly rather
than assuming double precision is a free pass forever.

## On the Hohmann → Propulsion integration

The original spec's example workflow, after a Hohmann transfer result,
offers three choices: calculate propellant, save the result, or return
to menu. This project implements the first and third. "Save result"
implies persistence - writing to a file, or at least holding onto past
results in memory - and there's no persistence layer anywhere in this
project. Rather than add a menu option that prints "not implemented
yet," which isn't meaningfully better than not having the option, only
the two choices that actually do something are offered. Result history
or saving to disk would be a real feature worth building properly if
it's ever needed, not a stub bolted on here.

## On error handling

The rocket equation is the first place in this project where "a number
that's technically parseable but not physically valid" needed more than
a simple range check. `propellant_required` and `achievable_delta_v`
both return `Result<PropulsionResult, RocketEquationError>` instead of
unwrapping or panicking - a negative or zero mass, a non-positive Isp, or
a final mass bigger than the initial mass all come back as a specific,
named error rather than a NaN or an infinity silently reaching the
results screen.

## Architecture notes

`HohmannTransfer` is its own struct rather than a method on `Orbit`,
since a transfer isn't a property of one orbit - it's a relationship
between two. It only needs the two circular radii and mu, the same
inputs `Orbit` already works with.

`propulsion` is a top-level module, separate from `physics`. Orbital
mechanics and Hohmann transfers describe what an orbit does independent
of anything flying through it; the rocket equation is entirely about the
vehicle. They're related (a transfer's Δv is the input to a propellant
calculation) but not the same kind of thing.

## Where this could go next

The original plan covered v0.1 through v0.5, ending with propulsion.
Beyond that, in no particular order, are directions this could grow in
if there's a reason to: mission design tools, numerical orbit
propagation, RK4 integration, three-dimensional orbits instead of the
current 2D model, additional central bodies besides Earth, and some kind
of visualization. None of this is planned - just the natural next steps
if the project keeps going.
