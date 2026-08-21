use crate::physics::body::CelestialBody;
use crate::physics::orbit::Orbit;
use crate::physics::orbit_type::OrbitType;
use crate::physics::vector2::Vector2;
use crate::ui;

/// Orbital Mechanics submenu. Three ways in now to define an orbit -
/// periapsis/apoapsis (from v0.2), semi-major axis/eccentricity, and
/// position/velocity vectors - all three end up at the same `Orbit`
/// and the same result screen.
pub fn menu_loop() {
    loop {
        ui::display::clear_screen();
        println!("ORBITAL MECHANICS\n");
        println!("1. Calculate Orbital Parameters");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => central_body_menu(),
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}

fn central_body_menu() {
    loop {
        ui::display::clear_screen();
        println!("CENTRAL BODY\n");
        println!("1. Earth");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => orbit_definition_menu(CelestialBody::earth()),
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}

fn orbit_definition_menu(body: CelestialBody) {
    loop {
        ui::display::clear_screen();
        println!("Central Body: {}\n", body.name);
        println!("Define orbit using:\n");
        println!("1. Periapsis & Apoapsis");
        println!("2. Semi-major axis & Eccentricity");
        println!("3. Position & Velocity");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => periapsis_apoapsis_flow(&body),
            2 => semi_major_axis_eccentricity_flow(&body),
            3 => position_velocity_flow(&body),
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}

// --- Method 1: Periapsis & Apoapsis ---

fn periapsis_apoapsis_flow(body: &CelestialBody) {
    ui::display::clear_screen();
    println!("PERIAPSIS & APOAPSIS\n");
    println!("Central Body: {}\n", body.name);

    let periapsis_alt_km = read_non_negative("Periapsis altitude [km]: ");
    let apoapsis_alt_km = read_non_negative("Apoapsis altitude [km]: ");

    if apoapsis_alt_km < periapsis_alt_km {
        println!("\nApoapsis can't be lower than periapsis.");
        ui::input::pause("Press ENTER to return...");
        return;
    }

    let radius_km = body.radius_m / 1000.0;
    let rp_km = radius_km + periapsis_alt_km;
    let ra_km = radius_km + apoapsis_alt_km;

    let orbit = Orbit::from_periapsis_apoapsis(rp_km * 1000.0, ra_km * 1000.0, body.gravitational_parameter);

    show_result(body, &orbit);
    ui::input::pause("\nPress ENTER to return...");
}

// --- Method 2: Semi-major axis & Eccentricity ---

fn semi_major_axis_eccentricity_flow(body: &CelestialBody) {
    ui::display::clear_screen();
    println!("SEMI-MAJOR AXIS & ECCENTRICITY\n");
    println!("Central Body: {}\n", body.name);

    let a_km = read_positive("Semi-major axis [km]: ");
    let e = read_eccentricity("Eccentricity: ");

    let orbit = Orbit::from_semi_major_axis_eccentricity(a_km * 1000.0, e, body.gravitational_parameter);

    if orbit.periapsis_radius_m() < body.radius_m {
        println!("\nThat semi-major axis and eccentricity put periapsis");
        println!("below {}'s surface - not a valid orbit.", body.name);
        ui::input::pause("Press ENTER to return...");
        return;
    }

    show_result(body, &orbit);
    ui::input::pause("\nPress ENTER to return...");
}

// --- Method 3: Position & Velocity ---

fn position_velocity_flow(body: &CelestialBody) {
    ui::display::clear_screen();
    println!("POSITION & VELOCITY\n");
    println!("Central Body: {}\n", body.name);
    println!("Position vector (from the center of {}):\n", body.name);

    let x_km = read_finite("x [km]: ");
    let y_km = read_finite("y [km]: ");

    println!("\nVelocity vector:\n");
    let vx_km_s = read_finite("vx [km/s]: ");
    let vy_km_s = read_finite("vy [km/s]: ");

    let position = Vector2::new(x_km * 1000.0, y_km * 1000.0);
    let velocity = Vector2::new(vx_km_s * 1000.0, vy_km_s * 1000.0);

    if position.magnitude() < body.radius_m {
        println!("\nThat position is inside {}'s surface.", body.name);
        ui::input::pause("Press ENTER to return...");
        return;
    }

    let orbit = Orbit::from_position_velocity(position, velocity, body.gravitational_parameter);
    show_result(body, &orbit);
    ui::input::pause("\nPress ENTER to return...");
}

fn read_non_negative(prompt: &str) -> f64 {
    loop {
        match ui::input::prompt(prompt).parse::<f64>() {
            Ok(v) if v >= 0.0 => return v,
            Ok(_) => println!("Can't be negative. Try again.\n"),
            Err(_) => println!("That's not a number. Try again.\n"),
        }
    }
}

fn read_positive(prompt: &str) -> f64 {
    loop {
        match ui::input::prompt(prompt).parse::<f64>() {
            Ok(v) if v > 0.0 => return v,
            Ok(_) => println!("Must be greater than zero. Try again.\n"),
            Err(_) => println!("That's not a number. Try again.\n"),
        }
    }
}

fn read_eccentricity(prompt: &str) -> f64 {
    loop {
        match ui::input::prompt(prompt).parse::<f64>() {
            Ok(v) if v >= 0.0 => return v,
            Ok(_) => println!("Eccentricity can't be negative. Try again.\n"),
            Err(_) => println!("That's not a number. Try again.\n"),
        }
    }
}

fn read_finite(prompt: &str) -> f64 {
    loop {
        match ui::input::prompt(prompt).parse::<f64>() {
            Ok(v) if v.is_finite() => return v,
            Ok(_) => println!("That's not a usable number. Try again.\n"),
            Err(_) => println!("That's not a number. Try again.\n"),
        }
    }
}

fn show_result(body: &CelestialBody, orbit: &Orbit) {
    let a_km = orbit.semi_major_axis_m / 1000.0;
    let rp_km = orbit.periapsis_radius_m() / 1000.0;
    let vp_km_s = orbit.velocity_at_periapsis_m_s() / 1000.0;
    let energy_mj_kg = orbit.specific_energy() / 1_000_000.0;
    let h_km2_s = orbit.specific_angular_momentum() / 1_000_000.0;

    println!("╔══════════════════════════════════════════════╗");
    println!("║                ORBIT RESULT                   ║");
    println!("╚══════════════════════════════════════════════╝\n");
    println!("Central Body: {}", body.name);
    println!("Orbit type:   {}\n", orbit.orbit_type());

    if orbit.semi_major_axis_m.is_finite() {
        println!("Semi-major axis:     {:>16.6} km", a_km);
    } else {
        println!("Semi-major axis:     {:>16}", "N/A (parabolic)");
    }
    println!("Eccentricity:        {:>16.8}", orbit.eccentricity);
    println!("Periapsis radius:    {:>16.6} km", rp_km);

    match orbit.apoapsis_radius_m() {
        Some(ra_m) => println!("Apoapsis radius:     {:>16.6} km", ra_m / 1000.0),
        None => println!("Apoapsis radius:     {:>16}", "N/A (unbound orbit)"),
    }

    println!("────────────────────────────────────────────────");

    match orbit.period_s() {
        Some(period_s) => {
            println!("Orbital period:      {:>16.6} min", period_s / 60.0);
            println!("                     {:>16.3} s", period_s);
        }
        None => println!("Orbital period:      {:>16}", "N/A (unbound orbit)"),
    }

    println!();
    println!("Velocity @ periapsis:{:>16.6} km/s", vp_km_s);
    match orbit.velocity_at_apoapsis_m_s() {
        Some(va_m_s) => println!("Velocity @ apoapsis: {:>16.6} km/s", va_m_s / 1000.0),
        None => println!("Velocity @ apoapsis: {:>16}", "N/A"),
    }

    println!();
    println!("Specific orbital energy: {:>16.6} MJ/kg", energy_mj_kg);
    println!("Specific angular momentum: {:.6} x 10^4 km^2/s", h_km2_s / 1e4);

    if orbit.orbit_type() == OrbitType::Hyperbolic {
        println!("\nNote: this trajectory escapes {} - it's a flyby,", body.name);
        println!("not a repeating orbit.");
    } else if orbit.orbit_type() == OrbitType::Parabolic {
        println!("\nNote: this is exactly escape trajectory - the");
        println!("theoretical boundary between orbiting and escaping.");
    }
}
