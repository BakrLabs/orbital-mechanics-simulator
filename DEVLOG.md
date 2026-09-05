# Dev Log

Notes from building this version by version. Not required reading to use
the tool - this is here for anyone curious about the "why" behind some of
the decisions, or checking my work on the physics. Organized by topic
rather than strictly by version, since the same themes (verification,
scope decisions, architecture) span several versions.

## Verification and numerical accuracy

Since this is meant to work as an actual calculator, it's worth being
specific about where the accuracy comes from, how it's been checked, and
where its limits are. This section pulls together every accuracy-related
finding across the project's history.

### The spec's worked examples

This project was built from a written spec (v0.1 through v0.5) with
worked numerical examples included, meant to double as expected test
output. Three of those examples didn't hold up when checked by hand:

**Periapsis/apoapsis velocities (v0.2).** For a 200km x 400km altitude
orbit around Earth, the spec's example gave periapsis/apoapsis velocity
as 7.784/7.669 km/s. Running the vis-viva equation - the same equation
the spec itself specifies - on that same orbit gives 7.842/7.611 km/s
instead. I checked this against angular momentum conservation (velocity
times radius should be equal at both periapsis and apoapsis, since
angular momentum doesn't change over the orbit) - it holds for 7.842/
7.611 and does not hold for 7.784/7.669. Semi-major axis, eccentricity,
period, and energy in that same example were all correct as given.

**Hohmann transfer delta-v (v0.4).** Same 200km-to-400km example. The
spec's individual burn figures (0.117 km/s and 0.114 km/s, 0.231 km/s
total) come out to almost exactly double what the stated vis-viva
formulas actually produce for that transfer (0.058 km/s and 0.058 km/s,
0.116 km/s total). The circular velocities, transfer semi-major axis,
and transfer time in the same example were all correct. As a second
check independent of the example entirely: total delta-v for a Hohmann
transfer should be identical whether you're raising an orbit or lowering
it between the same two altitudes, by the symmetry of the maneuver - my
numbers satisfy that property, the spec's original figures don't.

**Rocket equation (v0.5).** No discrepancy here - the spec's worked
example (delta-v = 9.40 km/s, Isp = 450s, m0 = 10000kg -> final mass ~
1188.285 kg) checked out cleanly against hand calculation.

In all three cases with a discrepancy, I went with what the stated
equations actually produce rather than the example's output, and added a
test that would fail if the calculation ever drifted toward the
incorrect example values instead.

### Closed-form accuracy (v0.1-v0.5)

**The equations are exact, not approximated** for everything through
v0.5. Vis-viva, Kepler's third law, the Hohmann transfer equations, and
the Tsiolkovsky rocket equation are all closed-form - no numerical
integration, no iterative solver, no truncated series. Given the
equations are correct (the LEO-to-GEO Hohmann transfer test matches the
commonly cited ~3.9 km/s figure, and GEO period comes out to ~23.93
hours, a sidereal day, as it should), the only source of error was
floating-point representation itself.

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

### RK4 propagation accuracy (v0.6)

`physics::propagator` steps a spacecraft's position and velocity forward
through time by numerically integrating the two-body equations of
motion (`a = -mu * r / |r|^3`) using classical 4th-order Runge-Kutta.
This is a genuinely different kind of calculation from the closed-form
work above - `Orbit` and `HohmannTransfer` evaluate equations at a
single instant; `Propagator` actually simulates motion step by step,
which raises a different accuracy question: not "is the formula exact"
but "how much does the numerical method's own error accumulate."

Before writing any Rust, I prototyped the same algorithm in Python and
checked:

- **Does a circular orbit return to its starting state after exactly one
  period?** At a 1-second step size, position error after a full orbit
  (500km altitude) came out under a micron. At a 30-second step size, it
  grew to about 1.4 meters - which is exactly the expected behavior for
  RK4 (error scales with step size to roughly the 4th power for a single
  step, and accumulates over many steps), not a sign of a bug.
- **Does the same hold for an eccentric orbit?** RK4's error is worst
  near periapsis, where curvature and acceleration are highest, so this
  is a harder case than circular. A 200km x 2000km orbit (e~0.12) still
  came back under 2 microns of position error at a 1-second step.
- **Do energy and angular momentum stay conserved during the run?** Both
  are supposed to be constant for an undisturbed two-body orbit - if they
  drift noticeably, that's the integrator's own numerical error showing
  up, not real physics. Tracked at every step in the test suite, and
  displayed to the user in the actual propagation results screen, not
  just checked internally.

All of that got carried over into the Rust implementation as unit tests
with the same tolerances, plus a bookkeeping test that a propagation
duration that doesn't divide evenly by the step size still lands exactly
on the requested total time (via a final partial step), rather than
silently propagating for slightly less time than asked.

### Stress-testing the fixed step size (post-v0.6)

A scope-limit note (see Architecture and the "where this could go next"
list) flagged fixed step size as a hypothetical concern. It's more
useful pinned down with an actual case where it bites.

Test setup: a highly eccentric orbit (perigee 200km altitude, apogee
about 1,006,700km, e ~ 0.987) propagated for one full period
(~3,587,171 seconds, about 41.5 days), at two fixed step sizes.

**dt = 60s.** The propagation completes without crashing, but the final
state lands about 2525 km away from where it started - it should have
returned almost exactly to its starting position after one full period.
Specific energy drifted by about -13.34 J/kg and specific angular
momentum by about -3868 m^2/s over the run, both of which should be
exactly conserved in an undisturbed two-body orbit. This step size is
too coarse for this orbit.

**dt = 10s.** Same orbit, same duration, only the step size changed. The
final position lands within about 0.84 km of the start (versus 2525 km),
energy drift drops to about -0.00086 J/kg, and angular momentum drift
to about -0.40 m^2/s - roughly a 3,000x improvement in position error
and a 15,000x improvement in energy drift from a 6x smaller step.

I reproduced every one of these figures independently before treating
them as correct, the same way spec examples get checked elsewhere in
this project. Both cases are now locked in as regression tests
(`highly_eccentric_orbit_60s_step_shows_expected_large_error` and
`highly_eccentric_orbit_10s_step_stays_tight` in `physics::propagator`),
so this exact behavior - the failure at 60s and the recovery at 10s -
can't silently regress.

The reason this orbit is such a harsh test: at e~0.987, the spacecraft
spends the overwhelming majority of its 41.5-day period crawling slowly
near apogee, then rips through perigee in minutes at high curvature and
speed. A fixed step size has to be small enough to resolve that fast
perigee pass, and then pays that same small step size for the entire
slow apogee coast too. This is the concrete case that made adaptive
step-size control (see below) worth actually doing.

### 3D orbital elements verification (v0.7)

The state-vector to orbital-elements conversion is real vector algebra
(cross products, a rotation matrix built from three Euler-angle-style
rotations) - more moving parts than anything built before it in this
project, so it got prototyped and round-trip tested in Python before any
Rust was written: start from a known set of elements, convert to a
position/velocity vector, convert back, and confirm the elements match
to floating-point precision. They did, cleanly, which is what gave
confidence to port the same logic into Rust as
`OrbitalElements::from_state_vector` and `to_state_vector`.

Beyond the round trip, the Rust test suite also checks:
- **Agreement with the existing 2D path.** An equatorial (i=0) circular
  orbit fed through the 3D code should produce the same semi-major axis
  and period as the same orbit fed through the existing 2D `Orbit` type.
  It does, which is the check that the new path isn't quietly
  inconsistent with the old one wherever they overlap.
- **A polar orbit** (velocity purely along the z-axis) comes back with
  inclination = 90 degrees, which is the textbook definition of a polar
  orbit and a good sanity check that the inclination formula's sign and
  axis conventions are right.

### Central body constants verification (v0.8)

`CelestialBody` gained `moon()`, `mars()`, and `sun()` alongside the
existing `earth()`, each with a real cited gravitational parameter and
radius (Moon: mu = 4.9028e12 m^3/s^2, radius 1738.09 km; Mars: mu =
4.282837e13 m^3/s^2, radius 3396.19 km; Sun: mu = 1.32712440042e20
m^3/s^2, radius 6.957e8 m). Each was cross-checked against a commonly
cited surface gravity figure (g = mu/r^2) before being trusted: Moon
~1.62 m/s^2, Mars ~3.71 m/s^2, Sun ~274 m/s^2 - all matched to within
the tolerance of the cited source values, and all three are now locked
in as unit tests in `physics::body`.

### 3D propagation verification (v0.8)

`physics::propagator_3d::Propagator3D` is the same RK4 machinery as the
2D `Propagator` from v0.6, generalized to `Vector3`. Verification
followed the same shape as everything else non-trivial in this project:
- **Planar case matches the 2D propagator exactly.** Propagating with
  z=0 and vz=0 through the 3D code should produce the same numbers as
  the existing 2D propagator for the same orbit. It does, to well
  beyond the precision either result is displayed at - this is the
  check that the new path isn't quietly different from the old one in
  the case where they should agree completely.
- **An inclined circular orbit keeps its orbital plane fixed.** For an
  undisturbed two-body orbit, the angular momentum vector's direction
  shouldn't change over time - only its magnitude is checked by the
  existing conservation tests, so this adds a direction check
  specifically, sampled throughout a full period at a 45-degree
  inclination. It holds to 1 part in 10,000 or better throughout the
  run.
- The existing 2D-style checks (circular orbit returns to start after
  one period, energy/angular momentum conservation) were ported over
  as their 3D equivalents too.

### Adaptive step-size control: verification and a known limitation (v0.9)

`Propagator3D::propagate_adaptive` adds RK4 with step-doubling: each
step is taken once at size `dt` and once as two steps at `dt/2`; the
difference between the two results estimates the local error. Too much
error and the step is rejected and retried smaller; comfortably under
and the next step grows; in between it stays put. This means the
integrator can take large steps through the slow part of an eccentric
orbit (near apoapsis) and small ones through the fast part (near
periapsis), instead of one fixed size paying the cost of the worst case
for the entire run - directly addressing the tradeoff the eccentric-
orbit stress test surfaced back in v0.6/v0.7.

This one got prototyped extensively in Python before any Rust was
written, because the first version of the tolerance-response curve
looked wrong: tightening the tolerance stopped helping past a point, and
even got worse at very tight settings. Before assuming that was a bug
worth chasing indefinitely, I ran a tolerance sweep on the known
eccentric-orbit case (the same 200km-perigee, e~0.987 orbit from the
v0.6/v0.7 stress test) and found a clean, explicable pattern:

| Tolerance (m) | Steps | Final position error |
|---|---|---|
| 1.0 | 1146 | 972.5 km |
| 0.1 | 1267 | 158.5 km |
| 0.01 | 1487 | 25.8 km |
| 0.001 | 1880 | 3.5 km |
| 0.0001 | 2578 | **93.9 m (best result)** |
| 0.00005 | 2836 | 387.4 m (worse) |
| 0.00001 | 3571 | 677.6 m (worse) |

Error improves smoothly and monotonically down to a tolerance around
0.0001 m, then gets *worse* as the tolerance tightens further. That's
floating-point noise, not a logic error: step-doubling's error estimate
is a difference between two position values that get closer and closer
together as `dt` shrinks, and at some point that subtraction is mostly
cancelling significant digits rather than measuring real truncation
error - the controller then starts reacting to numerical noise instead
of the actual local error, which is exactly the kind of thing that
produces a non-monotonic response. Position magnitudes in this problem
are around 10^7 m; `f64` carries roughly 15-16 significant digits, so a
few dozen arithmetic operations per RK4 step compounding their rounding
error landing in the 10^-4 m range is the right order of magnitude for
this, not a surprise.

Given that, the honest choice was to ship what actually works well
(anywhere from loose tolerances down to roughly 0.0001 m) with this
behavior documented and locked in as a test - not to chase a "perfect"
adaptive integrator that pretends floating-point arithmetic has infinite
precision. Fixing this properly would mean a smarter error norm (scaled
relative to the state's own magnitude, the standard approach in real
ODE solvers) or extended-precision arithmetic - either is a bigger,
separate piece of work, not a quick patch. The default tolerance used by
the elements-propagation integration (0.1 m) sits comfortably inside the
well-behaved range.

One more honest note on the numbers above: at a fixed tolerance,
adaptive stepping doesn't automatically beat every fixed step size - the
v0.6/v0.7 stress test's fixed 10-second step achieved 0.84 km of error
over the same orbit, better than adaptive's best (93.9 m is actually
better - but that comparison needed the tolerance sweep to find; the
default 0.1 m tolerance alone gives 158.5 km, worse than fixed 10s). The
real win is efficiency: adaptive's ~1267 steps at 0.1 m tolerance versus
fixed 10s's ~358,717 steps for accuracy in a similar range - not
"always more accurate," but "comparable accuracy for far less
computation," which is the actual point of adaptive step-size control.

### Switching to relative tolerance (v1.0)

The v0.9 section above used absolute position-error tolerance in
meters. Before hardening this for a 1.0 release, I looked at how real
orbit propagation tools actually do this - GMAT and STK both use
*relative* error tolerance (typically in the 1e-9 to 1e-12 range),
scaled by the magnitude of the state being integrated, not a fixed
distance. Orekit's documented default (`dP = 0.001` m) is absolute, but
its own docs explicitly warn that a millimeter tolerance setting doesn't
guarantee millimeter final accuracy - the same caveat this project
already carries.

Absolute-meters tolerance has a real problem this project's own multi-
body support exposes directly: "0.1 meters" means something completely
different for a orbit a few thousand km across (Earth or Moon) than for
one hundreds of millions of km across (a solar orbit). A tolerance
tuned for Earth orbits either does far more work than necessary for a
solar orbit, or isn't tight enough, depending on which way the mismatch
goes. Relative tolerance - error as a fraction of the current position
magnitude - means the same tolerance value behaves consistently no
matter which of the four central bodies is in play.

`Propagator3D::propagate_adaptive` now takes a dimensionless
`relative_tolerance` instead of `tolerance_m`, and
`step_with_error_estimate` divides the step-doubling absolute error by
the current position magnitude before comparing against it. I verified
this actually delivers on the promise before trusting it: propagating a
close orbit around the Moon (radius ~1.8 million meters) and around the
Sun (radius ~730 million meters - about 400x larger) at the identical
relative tolerance of 1e-9 converges in essentially the same number of
steps (300 vs 299) for each. Under the old absolute-meters scheme this
comparison wouldn't have been meaningful at all. This is now a locked-in
test (`relative_tolerance_behaves_consistently_across_wildly_different_orbit_scales`).

The floating-point noise floor from v0.9 didn't disappear - it moved.
Re-running the same eccentric-orbit tolerance sweep with relative error
instead of absolute:

| Relative tolerance | Steps | Final position error |
|---|---|---|
| 1e-8 | 1192 | 204.8 km |
| 1e-9 | 1336 | 34.7 km |
| 1e-10 | 1581 | 5.3 km |
| 1e-11 | 2001 | 207.5 m (near best) |
| 1e-12 | 2711 | 626.5 m (worse) |
| 1e-13 | 3668 | 763.4 m (worse) |
| 1e-14 | 5135 | 785.7 m (worse) |

Same shape as before - monotonic improvement, then a floor, then
regression - just at a different crossover point (around 1e-11 relative
instead of 1e-4 m absolute). That confirms this is a property of
step-doubling error estimation at `f64` precision in general, not an
artifact of the specific tolerance representation chosen in v0.9. The
default tolerance for the elements-propagation integration is now 1e-9,
comfortably inside the well-behaved range and matching the low end of
what GMAT/STK commonly use in practice.

## Consistency pass across central bodies (v1.0)

Before calling this a stable release, I checked whether Moon/Mars/Sun
(added in v0.8) actually behave correctly through every flow that
accepts a central body, not just Earth, which has had the most
iteration. Found no bugs, but it's worth recording what was checked:

- Every "is this position inside the body's surface" check compares
  against `body.radius_m`, which is body-specific - none of them were
  accidentally hardcoded to Earth's radius.
- Orbit type classification (`OrbitType::from_eccentricity`) and the 3D
  orbit near-planar/near-polar detection in the results screen both use
  dimensionless thresholds (eccentricity, angle in degrees) - neither
  depends on distance scale, so they're correct for any body by
  construction, not by luck.
- The relative-tolerance adaptive propagator was specifically checked
  across the largest scale gap in the app (Moon vs. Sun, ~400x
  difference in orbital radius) and behaves consistently - see above.
- No hardcoded "Earth" string or Earth-specific numeric constant remains
  in any app-layer flow (`orbital_mechanics.rs`, `hohmann_transfer.rs`,
  `propulsion.rs`, `orbital_elements_3d.rs`, `propagation.rs`) - all
  central-body dependence goes through the `CelestialBody` struct.

## Feature notes

### The Hohmann to Propulsion integration (v0.5)

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

### Error handling (v0.5, v0.6)

The rocket equation (v0.5) is the first place in this project where "a
number that's technically parseable but not physically valid" needed
more than a simple range check. `propellant_required` and
`achievable_delta_v` both return `Result<PropulsionResult,
RocketEquationError>` instead of unwrapping or panicking - a negative or
zero mass, a non-positive Isp, or a final mass bigger than the initial
mass all come back as a specific, named error rather than a NaN or an
infinity silently reaching the results screen.

The propagation flow (v0.6) applies the same principle to step size:
a step size longer than the total propagation duration is rejected with
a message rather than silently accepted (the underlying propagator would
technically handle it via its partial-step logic, but "one step longer
than the whole run" isn't a meaningful step size to allow through).

### 3D orbital elements as a parallel path, not a migration (v0.7)

Everything through v0.6 was quietly planar - `Vector2`, `Orbit`, and the
propagator all assume the spacecraft's motion stays in a single 2D
plane. Real orbits have inclination, and don't generally share a plane
with each other, so v0.7 adds the full six classical (Keplerian) orbital
elements: semi-major axis and eccentricity (shape, already had these),
inclination and RAAN (orientation of the orbital plane), argument of
periapsis (orientation of the ellipse within that plane), and true
anomaly (where the spacecraft currently is).

`physics::orbital_elements` and `physics::vector3` are new modules that
sit alongside the existing `physics::orbit` and `physics::vector2`
rather than replacing them. The alternative - migrating everything to 3D
under the hood, with the old 2D behavior as the special case where
inclination is zero - would have been a more unified design, but also a
much bigger refactor touching every physics module and every test that
currently passes. Keeping them separate means nothing that already
worked had to be touched to add this.

Scope limit at the time: propagation (`physics::propagator`, from v0.6)
was still 2D-only. Extending RK4 to 3D turned out to be a smaller lift
than the orbital elements work itself (the equations of motion
generalize directly - it's really just adding a z-component throughout),
but it was still a distinct piece of work with its own tests, done in
v0.8 instead.

### Additional central bodies (v0.8)

Every menu that takes a central body (Orbital Mechanics, Hohmann
Transfer, Orbit Propagation) offers all four (Earth, Moon, Mars, Sun),
through one shared `app::central_body::select()` function rather than
four separate copies of the same "1. Earth, 0. Back" menu. Adding a
fifth body later is a one-line change in `CelestialBody` plus one line
in that shared menu, not a hunt through four files.

Central bodies and 3D propagation went into the same version because
they reinforce each other more than either does alone: a 3D orbit around
a body other than Earth is a much better demonstration of "this actually
generalizes" than either piece in isolation.

The propagation app flow offers both 2D and 3D as separate menu options
rather than only exposing 3D once it exists - 2D propagation is simpler
to set up when the orbit really is planar, so there wasn't a reason to
make 3D mandatory once the extra z-component was on the table.

### 3D Elements to Propagation integration (v0.9)

After defining a 3D orbit (either method - direct elements or state
vector), the results screen now offers "Propagate this orbit for one
period," which calls `OrbitalElements::to_state_vector()` (already built
in v0.7) and feeds the result straight into `Propagator3D`, the same
pattern Hohmann Transfer uses to hand off to Propulsion. Unbound orbits
(parabolic or hyperbolic) don't have a period to propagate for, so that
path prints an explanation and points at Orbit Propagation directly
instead of pretending a period exists.

This was bundled with adaptive step-size control in the same version
because connecting 3D Orbital Elements to Orbit Propagation is more
useful once propagation handles a wide range of orbits well without the
user hand-tuning a fixed step size for each one.

## Architecture notes

`HohmannTransfer` is its own struct rather than a method on `Orbit`,
since a transfer isn't a property of one orbit - it's a relationship
between two. It only needs the two circular radii and mu, the same
inputs `Orbit` already works with.

`propulsion` is a top-level module, separate from `physics`. Orbital
mechanics and Hohmann transfers describe what an orbit does independent
of anything flying through it; the rocket equation is entirely about the
vehicle. They're related (a transfer's delta-v is the input to a
propellant calculation) but not the same kind of thing.

`propagator` lives inside `physics`, not as its own top-level module
like `propulsion` did - unlike the rocket equation, propagation is
still squarely about the orbit itself (where the spacecraft ends up),
just calculated a different way than `Orbit`'s closed-form equations.

## Where this could go next

This project reached `v1.0` as a deliberate hardening release rather
than a new-feature version: the goal was consistency and correctness
across what already existed (relative-tolerance adaptive propagation,
a verified cross-body consistency pass), not new capability. Two real
feature directions remain open and are explicitly deferred past 1.0,
not half-built:

- **Mission design tools** - chaining maneuvers, patched conics. Not
  started. A meaningfully large feature in its own right, deliberately
  left for a future major version rather than attempted piecemeal.
- **Visualization** - some way to see an orbit, not just read numbers
  off it. Not started, same reasoning as above.

Smaller, lower-priority items:
- **An even more scale-invariant error norm** for the adaptive
  propagator (e.g. combining relative position and velocity error,
  which real ODE solvers often do) could push accuracy a bit further,
  but the current relative-tolerance approach (v1.0) already resolves
  the practical cross-body consistency problem this project cares
  about.
- **Additional central bodies** beyond the current four remain easy to
  add (one function plus one menu line) whenever there's a reason to.

Resolved since first noted:
- ~~3D orbits~~ - done in v0.7 (orbital elements) and v0.8 (propagation).
- ~~Additional central bodies~~ - done in v0.8 (Moon, Mars, Sun).
- ~~Adaptive step-size control~~ - done in v0.9, refined to relative
  tolerance in v1.0, with a documented floating-point limitation at very
  tight tolerances either way (see Verification section above).
- ~~3D elements/propagation integration~~ - done in v0.9.
- ~~Absolute-vs-relative tolerance~~ - resolved in v1.0; see above.
