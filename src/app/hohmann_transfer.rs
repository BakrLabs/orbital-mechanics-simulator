use crate::app::central_body;
use crate::app::propulsion;
use crate::physics::body::CelestialBody;
use crate::physics::hohmann::HohmannTransfer;
use crate::ui;

pub fn menu_loop() {
    if let Some(body) = central_body::select() {
        transfer_flow(body);
    }
}

fn transfer_flow(body: CelestialBody) {
    ui::display::clear_screen();
    println!("HOHMANN TRANSFER\n");
    println!("Central Body: {}\n", body.name);
    println!("Both orbits are assumed circular.\n");

    let initial_alt_km = read_non_negative("Initial orbit altitude [km]: ");
    let target_alt_km = read_non_negative("Target orbit altitude [km]: ");

    let radius_km = body.radius_m / 1000.0;
    let r1_km = radius_km + initial_alt_km;
    let r2_km = radius_km + target_alt_km;

    if initial_alt_km == target_alt_km {
        println!("\nInitial and target altitude are the same - nothing to transfer.");
        ui::input::pause("Press ENTER to return...");
        return;
    }

    let transfer = HohmannTransfer::new(r1_km * 1000.0, r2_km * 1000.0, body.gravitational_parameter);

    show_result(&body, initial_alt_km, target_alt_km, r1_km, r2_km, &transfer);
    post_result_menu(&transfer);
}

// no persistence layer exists yet, so unlike the spec's example menu
// this only offers propellant calc + return, not a "save result"
// option that wouldn't actually do anything
fn post_result_menu(transfer: &HohmannTransfer) {
    loop {
        println!("\nWhat would you like to do?\n");
        println!("1. Calculate required propellant");
        println!("2. Return to menu");

        match ui::input::read_menu_choice("\nSelect: ") {
            1 => {
                propulsion::propellant_for_delta_v(transfer.total_delta_v_m_s());
                return;
            }
            2 => return,
            _ => {
                println!("\nNot a valid option.");
            }
        }
    }
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

fn show_result(
    body: &CelestialBody,
    initial_alt_km: f64,
    target_alt_km: f64,
    r1_km: f64,
    r2_km: f64,
    transfer: &HohmannTransfer,
) {
    let v1_km_s = transfer.initial_circular_velocity_m_s() / 1000.0;
    let v2_km_s = transfer.final_circular_velocity_m_s() / 1000.0;
    let at_km = transfer.transfer_semi_major_axis_m() / 1000.0;
    let dv1_km_s = transfer.burn1_delta_v_m_s() / 1000.0;
    let dv2_km_s = transfer.burn2_delta_v_m_s() / 1000.0;
    let total_dv_km_s = transfer.total_delta_v_m_s() / 1000.0;
    let transfer_time_min = transfer.transfer_time_s() / 60.0;

    println!("╔══════════════════════════════════════════════╗");
    println!("║               HOHMANN TRANSFER               ║");
    println!("╚══════════════════════════════════════════════╝\n");
    println!("{}\n", body.name);

    println!("INITIAL ORBIT");
    println!("{:<22}{:>14.6} km", "Altitude:", initial_alt_km);
    println!("{:<22}{:>14.6} km", "Radius:", r1_km);
    println!("{:<22}{:>14.6} km/s\n", "Circular velocity:", v1_km_s);

    println!("TARGET ORBIT");
    println!("{:<22}{:>14.6} km", "Altitude:", target_alt_km);
    println!("{:<22}{:>14.6} km", "Radius:", r2_km);
    println!("{:<22}{:>14.6} km/s\n", "Circular velocity:", v2_km_s);

    println!("TRANSFER ORBIT");
    println!("{:<22}{:>14.6} km", "Semi-major axis:", at_km);

    println!("\n────────────────────────────────────────────────\n");

    println!("BURN #1");
    println!("{:<22}{:>14.6} km/s   ({:>10.3} m/s)\n", "Δv:", dv1_km_s, dv1_km_s * 1000.0);

    println!("BURN #2");
    println!("{:<22}{:>14.6} km/s   ({:>10.3} m/s)\n", "Δv:", dv2_km_s, dv2_km_s * 1000.0);

    println!("TOTAL Δv:");
    println!("{:<22}{:>14.6} km/s   ({:>10.3} m/s)\n", "", total_dv_km_s, total_dv_km_s * 1000.0);

    println!("TRANSFER TIME:");
    println!("{:<22}{:>14.6} minutes", "", transfer_time_min);
    println!("{:<22}{:>14.6} seconds", "", transfer.transfer_time_s());

    if target_alt_km < initial_alt_km {
        println!("\nNote: target orbit is lower - both burns are retrograde");
        println!("(this lowers the orbit rather than raising it).");
    }
}
