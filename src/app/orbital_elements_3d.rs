use crate::app::propagation;
use crate::physics::body::CelestialBody;
use crate::physics::orbital_elements::OrbitalElements;
use crate::physics::vector3::Vector3;
use crate::ui;

pub fn elements_flow(body: &CelestialBody) {
    ui::display::clear_screen();
    println!("3D ORBITAL ELEMENTS\n");
    println!("Central Body: {}\n", body.name);

    let a_km = read_positive("Semi-major axis [km]: ");
    let e = read_eccentricity("Eccentricity: ");
    let i_deg = read_finite("Inclination [deg]: ");
    let raan_deg = read_finite("RAAN [deg]: ");
    let argp_deg = read_finite("Argument of periapsis [deg]: ");
    let nu_deg = read_finite("True anomaly [deg]: ");

    let elements = OrbitalElements::new(
        a_km * 1000.0,
        e,
        i_deg.to_radians(),
        raan_deg.to_radians(),
        argp_deg.to_radians(),
        nu_deg.to_radians(),
        body.gravitational_parameter,
    );

    show_result(body, &elements);
    post_result_menu(body, &elements);
}

pub fn position_velocity_3d_flow(body: &CelestialBody) {
    ui::display::clear_screen();
    println!("3D POSITION & VELOCITY\n");
    println!("Central Body: {}\n", body.name);
    println!("Position vector (from the center of {}):\n", body.name);

    let x_km = read_finite("x [km]: ");
    let y_km = read_finite("y [km]: ");
    let z_km = read_finite("z [km]: ");

    println!("\nVelocity vector:\n");
    let vx_km_s = read_finite("vx [km/s]: ");
    let vy_km_s = read_finite("vy [km/s]: ");
    let vz_km_s = read_finite("vz [km/s]: ");

    let position = Vector3::new(x_km * 1000.0, y_km * 1000.0, z_km * 1000.0);
    let velocity = Vector3::new(vx_km_s * 1000.0, vy_km_s * 1000.0, vz_km_s * 1000.0);

    if position.magnitude() < body.radius_m {
        println!("\nThat position is inside {}'s surface.", body.name);
        ui::input::pause("Press ENTER to return...");
        return;
    }

    let elements = OrbitalElements::from_state_vector(position, velocity, body.gravitational_parameter);
    show_result(body, &elements);
    post_result_menu(body, &elements);
}

// mirrors the Hohmann Transfer -> Propulsion pattern: this orbit's
// state vector goes straight into a propagation instead of the user
// re-typing six numbers they already just entered
fn post_result_menu(body: &CelestialBody, elements: &OrbitalElements) {
    loop {
        println!("\nWhat would you like to do?\n");
        println!("1. Propagate this orbit for one period");
        println!("2. Return to menu");

        match ui::input::read_menu_choice("\nSelect: ") {
            1 => {
                match elements.period_s() {
                    Some(period_s) => {
                        let (position, velocity) = elements.to_state_vector();
                        propagation::propagate_orbit_for_duration(body, position, velocity, period_s);
                    }
                    None => {
                        println!("\nThis orbit is unbound (parabolic or hyperbolic) - there's");
                        println!("no period to propagate for. Use Orbit Propagation directly");
                        println!("with a specific duration instead.");
                        ui::input::pause("Press ENTER to continue...");
                    }
                }
                return;
            }
            2 => return,
            _ => {
                println!("\nNot a valid option.");
            }
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

fn show_result(body: &CelestialBody, elements: &OrbitalElements) {
    println!("╔══════════════════════════════════════════════╗");
    println!("║          3D ORBITAL ELEMENTS RESULT          ║");
    println!("╚══════════════════════════════════════════════╝\n");
    println!("Central Body: {}\n", body.name);

    println!("{:<26}{:>14.6} km", "Semi-major axis:", elements.semi_major_axis_m / 1000.0);
    println!("{:<26}{:>14.8}", "Eccentricity:", elements.eccentricity);
    println!("{:<26}{:>14.6} deg", "Inclination:", elements.inclination_deg());
    println!("{:<26}{:>14.6} deg", "RAAN:", elements.raan_deg());
    println!("{:<26}{:>14.6} deg", "Argument of periapsis:", elements.argument_of_periapsis_deg());
    println!("{:<26}{:>14.6} deg", "True anomaly:", elements.true_anomaly_deg());

    println!("\n────────────────────────────────────────────────\n");

    match elements.period_s() {
        Some(period_s) => {
            println!("{:<26}{:>14.6} min", "Orbital period:", period_s / 60.0);
        }
        None => println!("{:<26}{:>14}", "Orbital period:", "N/A (unbound orbit)"),
    }

    if elements.inclination_deg().abs() < 0.01 {
        println!("\nNote: inclination is ~0 - this orbit lies in the reference");
        println!("plane and could equally be handled by the 2D methods.");
    } else if (elements.inclination_deg() - 90.0).abs() < 0.01 {
        println!("\nNote: this is a polar orbit (inclination ~90 deg).");
    }
}
