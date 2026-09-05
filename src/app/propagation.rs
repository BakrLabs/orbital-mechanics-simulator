use crate::app::central_body;
use crate::physics::body::CelestialBody;
use crate::physics::propagator::{PropagatedState, Propagator};
use crate::physics::propagator_3d::{PropagatedState3D, Propagator3D};
use crate::physics::vector2::Vector2;
use crate::physics::vector3::Vector3;
use crate::ui;

pub fn menu_loop() {
    loop {
        ui::display::clear_screen();
        println!("ORBIT PROPAGATION\n");
        println!("1. Propagate a State Vector (2D, fixed step)");
        println!("2. Propagate a State Vector (3D, fixed step)");
        println!("3. Propagate a State Vector (3D, adaptive step)");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => {
                if let Some(body) = central_body::select() {
                    propagation_flow(body);
                }
            }
            2 => {
                if let Some(body) = central_body::select() {
                    propagation_flow_3d(body);
                }
            }
            3 => {
                if let Some(body) = central_body::select() {
                    propagation_flow_3d_adaptive(body);
                }
            }
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}

// entry point for the 3D Orbital Elements "propagate this orbit" flow - uses the documented default tolerance
const DEFAULT_ADAPTIVE_RELATIVE_TOLERANCE: f64 = 1e-9;
const DEFAULT_ADAPTIVE_INITIAL_DT_S: f64 = 60.0;

pub fn propagate_orbit_for_duration(body: &CelestialBody, position: Vector3, velocity: Vector3, duration_s: f64) {
    ui::display::clear_screen();
    println!("PROPAGATE ORBIT (adaptive, relative tolerance = {DEFAULT_ADAPTIVE_RELATIVE_TOLERANCE:e})\n");
    println!("Central Body: {}\n", body.name);

    let propagator = Propagator3D::new(body.gravitational_parameter);
    let states = propagator.propagate_adaptive(
        position,
        velocity,
        duration_s,
        DEFAULT_ADAPTIVE_RELATIVE_TOLERANCE,
        DEFAULT_ADAPTIVE_INITIAL_DT_S,
    );

    show_result_3d(body, &states);
    println!("\n(Adaptive run used {} steps.)", states.len());
    ui::input::pause("\nPress ENTER to return...");
}

fn propagation_flow(body: CelestialBody) {
    ui::display::clear_screen();
    println!("PROPAGATE A STATE VECTOR\n");
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

    println!();
    let duration_s = read_positive("Propagation duration [s]: ");
    let step_s = read_step_size("Time step [s]: ", duration_s);

    let propagator = Propagator::new(body.gravitational_parameter);
    let states = propagator.propagate(position, velocity, duration_s, step_s);

    show_result(&body, &states);
    ui::input::pause("\nPress ENTER to return...");
}

fn propagation_flow_3d(body: CelestialBody) {
    ui::display::clear_screen();
    println!("PROPAGATE A STATE VECTOR (3D)\n");
    println!("Central Body: {}\n", body.name);

    let (position, velocity) = match read_3d_state(&body) {
        Some(state) => state,
        None => return,
    };

    println!();
    let duration_s = read_positive("Propagation duration [s]: ");
    let step_s = read_step_size("Time step [s]: ", duration_s);

    let propagator = Propagator3D::new(body.gravitational_parameter);
    let states = propagator.propagate(position, velocity, duration_s, step_s);

    show_result_3d(&body, &states);
    ui::input::pause("\nPress ENTER to return...");
}

fn propagation_flow_3d_adaptive(body: CelestialBody) {
    ui::display::clear_screen();
    println!("PROPAGATE A STATE VECTOR (3D, ADAPTIVE STEP)\n");
    println!("Central Body: {}\n", body.name);

    let (position, velocity) = match read_3d_state(&body) {
        Some(state) => state,
        None => return,
    };

    println!();
    let duration_s = read_positive("Propagation duration [s]: ");
    println!("\nRelative error tolerance per step (dimensionless, scaled by");
    println!("current position magnitude - the same convention real orbit");
    println!("propagation tools like GMAT/STK use). Smaller means more");
    println!("accurate but more steps. 1e-9 is a solid default; going");
    println!("much tighter than ~1e-11 stops helping (see About/DEVLOG).");
    let relative_tolerance = read_positive("Relative tolerance (e.g. 1e-9): ");
    let initial_dt_s = read_positive("Initial time step guess [s]: ");

    let propagator = Propagator3D::new(body.gravitational_parameter);
    let states = propagator.propagate_adaptive(position, velocity, duration_s, relative_tolerance, initial_dt_s);

    show_result_3d(&body, &states);
    println!("\n(Adaptive run used {} steps.)", states.len());
    ui::input::pause("\nPress ENTER to return...");
}

// shared by both 3D propagation flows - reads position/velocity and
// rejects a position inside the body's surface
fn read_3d_state(body: &CelestialBody) -> Option<(Vector3, Vector3)> {
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
        return None;
    }

    Some((position, velocity))
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

// one "step" longer than the whole run isn't a real step size
fn read_step_size(prompt: &str, duration_s: f64) -> f64 {
    loop {
        match ui::input::prompt(prompt).parse::<f64>() {
            Ok(v) if v > 0.0 && v <= duration_s => return v,
            Ok(v) if v > duration_s => {
                println!("Step size can't be longer than the propagation duration. Try again.\n")
            }
            Ok(_) => println!("Must be greater than zero. Try again.\n"),
            Err(_) => println!("That's not a number. Try again.\n"),
        }
    }
}

fn show_result(body: &CelestialBody, states: &[PropagatedState]) {
    let initial = states.first().unwrap();
    let final_state = states.last().unwrap();

    println!("╔══════════════════════════════════════════════╗");
    println!("║              PROPAGATION RESULT              ║");
    println!("╚══════════════════════════════════════════════╝\n");
    println!("Central Body: {}", body.name);
    println!("Total propagated time: {:.3} s\n", final_state.elapsed_time_s);

    println!("FINAL STATE");
    println!(
        "Position:  x = {:>16.6} km   y = {:>16.6} km",
        final_state.position_m.x / 1000.0,
        final_state.position_m.y / 1000.0
    );
    println!(
        "Velocity: vx = {:>16.6} km/s vy = {:>16.6} km/s",
        final_state.velocity_m_s.x / 1000.0,
        final_state.velocity_m_s.y / 1000.0
    );
    println!("Radius:{:>25.6} km", final_state.position_m.magnitude() / 1000.0);
    println!("Speed: {:>25.6} km/s\n", final_state.velocity_m_s.magnitude() / 1000.0);

    println!("CONSERVATION CHECK");
    println!("(Specific energy and angular momentum should stay constant");
    println!("through an undisturbed two-body orbit - this shows how");
    println!("closely the numerical integration holds that.)\n");

    print_conservation_table(states);

    let energy_drift = final_state.specific_energy - initial.specific_energy;
    let h_drift = final_state.specific_angular_momentum - initial.specific_angular_momentum;
    println!("\n{:<32}{:>12.6e} J/kg", "Total energy drift:", energy_drift);
    println!("{:<32}{:>12.6e} m^2/s", "Total angular momentum drift:", h_drift);
}

// caps at 10 rows so a small step size doesn't dump thousands of lines
fn print_conservation_table(states: &[PropagatedState]) {
    println!(
        "{:>12}  {:>18}  {:>18}",
        "Time [s]", "Energy [MJ/kg]", "Ang. Mom [km^2/s]"
    );
    println!("{}", "-".repeat(54));

    let max_rows = 10;
    let step_count = states.len();
    let row_interval = if step_count <= max_rows {
        1
    } else {
        step_count / (max_rows - 1)
    };

    for (i, state) in states.iter().enumerate() {
        let is_last = i == states.len() - 1;
        if i % row_interval == 0 || is_last {
            println!(
                "{:>12.2}  {:>18.9}  {:>18.6}",
                state.elapsed_time_s,
                state.specific_energy / 1_000_000.0,
                state.specific_angular_momentum / 1_000_000.0
            );
        }
    }
}

fn show_result_3d(body: &CelestialBody, states: &[PropagatedState3D]) {
    let initial = states.first().unwrap();
    let final_state = states.last().unwrap();

    println!("╔══════════════════════════════════════════════╗");
    println!("║           PROPAGATION RESULT (3D)            ║");
    println!("╚══════════════════════════════════════════════╝\n");
    println!("Central Body: {}", body.name);
    println!("Total propagated time: {:.3} s\n", final_state.elapsed_time_s);

    println!("FINAL STATE");
    println!(
        "Position:  x = {:>14.6}  y = {:>14.6}  z = {:>14.6} km",
        final_state.position_m.x / 1000.0,
        final_state.position_m.y / 1000.0,
        final_state.position_m.z / 1000.0
    );
    println!(
        "Velocity: vx = {:>14.6} vy = {:>14.6} vz = {:>14.6} km/s",
        final_state.velocity_m_s.x / 1000.0,
        final_state.velocity_m_s.y / 1000.0,
        final_state.velocity_m_s.z / 1000.0
    );
    println!("Radius:{:>25.6} km", final_state.position_m.magnitude() / 1000.0);
    println!("Speed: {:>25.6} km/s\n", final_state.velocity_m_s.magnitude() / 1000.0);

    println!("CONSERVATION CHECK");
    println!("(Specific energy and angular momentum should stay constant");
    println!("through an undisturbed two-body orbit - this shows how");
    println!("closely the numerical integration holds that.)\n");

    print_conservation_table_3d(states);

    let energy_drift = final_state.specific_energy - initial.specific_energy;
    let h_drift = final_state.specific_angular_momentum - initial.specific_angular_momentum;
    println!("\n{:<32}{:>12.6e} J/kg", "Total energy drift:", energy_drift);
    println!("{:<32}{:>12.6e} m^2/s", "Total angular momentum drift:", h_drift);
}

fn print_conservation_table_3d(states: &[PropagatedState3D]) {
    println!(
        "{:>12}  {:>18}  {:>18}",
        "Time [s]", "Energy [MJ/kg]", "Ang. Mom [km^2/s]"
    );
    println!("{}", "-".repeat(54));

    let max_rows = 10;
    let step_count = states.len();
    let row_interval = if step_count <= max_rows {
        1
    } else {
        step_count / (max_rows - 1)
    };

    for (i, state) in states.iter().enumerate() {
        let is_last = i == states.len() - 1;
        if i % row_interval == 0 || is_last {
            println!(
                "{:>12.2}  {:>18.9}  {:>18.6}",
                state.elapsed_time_s,
                state.specific_energy / 1_000_000.0,
                state.specific_angular_momentum / 1_000_000.0
            );
        }
    }
}
