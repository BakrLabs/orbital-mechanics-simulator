use crate::propulsion::rocket_equation::{self, PropulsionResult};
use crate::ui;

pub fn menu_loop() {
    loop {
        ui::display::clear_screen();
        println!("PROPULSION\n");
        println!("1. Calculate Propellant Required");
        println!("2. Calculate Achievable Δv");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => propellant_required_flow(),
            2 => achievable_delta_v_flow(),
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}

fn propellant_required_flow() {
    ui::display::clear_screen();
    println!("PROPELLANT REQUIRED\n");

    let delta_v_km_s = read_finite("Required Δv [km/s]: ");
    let isp_s = read_finite("Specific impulse [s]: ");
    let initial_mass_kg = read_finite("Initial mass [kg]: ");

    match rocket_equation::propellant_required(delta_v_km_s * 1000.0, isp_s, initial_mass_kg) {
        Ok(result) => show_result(&result),
        Err(e) => {
            println!("\n{}", e);
        }
    }

    ui::input::pause("\nPress ENTER to return...");
}

fn achievable_delta_v_flow() {
    ui::display::clear_screen();
    println!("ACHIEVABLE Δv\n");

    let isp_s = read_finite("Specific impulse [s]: ");
    let initial_mass_kg = read_finite("Initial mass [kg]: ");
    let final_mass_kg = read_finite("Final mass [kg]: ");

    match rocket_equation::achievable_delta_v(isp_s, initial_mass_kg, final_mass_kg) {
        Ok(result) => show_result(&result),
        Err(e) => {
            println!("\n{}", e);
        }
    }

    ui::input::pause("\nPress ENTER to return...");
}

// called from the Hohmann transfer flow once it has a delta-v to
// spend propellant on
pub fn propellant_for_delta_v(delta_v_m_s: f64) {
    ui::display::clear_screen();
    println!("PROPELLANT REQUIRED\n");
    println!("Hohmann transfer Δv: {:.6} km/s\n", delta_v_m_s / 1000.0);

    let isp_s = read_finite("Engine Isp [s]: ");
    let initial_mass_kg = read_finite("Spacecraft initial mass [kg]: ");

    match rocket_equation::propellant_required(delta_v_m_s, isp_s, initial_mass_kg) {
        Ok(result) => show_result(&result),
        Err(e) => {
            println!("\n{}", e);
        }
    }

    ui::input::pause("\nPress ENTER to return...");
}

// negative values pass through here fine - propellant_required/
// achievable_delta_v are what actually decide valid ranges and hand
// back a proper error, so validation only lives in one place
fn read_finite(prompt: &str) -> f64 {
    loop {
        match ui::input::prompt(prompt).parse::<f64>() {
            Ok(v) if v.is_finite() => return v,
            Ok(_) => println!("That's not a usable number. Try again.\n"),
            Err(_) => println!("That's not a number. Try again.\n"),
        }
    }
}

fn show_result(result: &PropulsionResult) {
    println!("\n╔══════════════════════════════════════════════╗");
    println!("║              PROPULSION RESULT               ║");
    println!("╚══════════════════════════════════════════════╝\n");

    println!("{:<22}{:>14.6} km/s", "Required Δv:", result.delta_v_m_s / 1000.0);
    println!("{:<22}{:>14.6} s", "Specific impulse:", result.specific_impulse_s);
    println!("{:<22}{:>14.6} m/s", "Exhaust velocity:", result.exhaust_velocity_m_s());
    println!("{:<22}{:>14.6} kg\n", "Initial mass:", result.initial_mass_kg);

    println!("{:<22}{:>14.6} kg", "Final mass:", result.final_mass_kg);
    println!("{:<22}{:>14.6} kg\n", "Propellant mass:", result.propellant_mass_kg());

    println!("{:<22}{:>14.6}", "Mass ratio:", result.mass_ratio());
}
