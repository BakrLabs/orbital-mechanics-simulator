use crate::app::propulsion;
use crate::physics::body::CelestialBody;
use crate::physics::hohmann::HohmannTransfer;
use crate::ui;

/// Hohmann Transfer submenu. Circular-to-circular only for now - an
/// elliptical starting or ending orbit needs to pick a specific point
/// on that ellipse to transfer from/to, which is a bigger feature than
/// this version is trying to be.
pub fn menu_loop() {
    loop {
        ui::display::clear_screen();
        println!("HOHMANN TRANSFER\n");
        println!("Central Body:");
        println!("1. Earth");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => transfer_flow(CelestialBody::earth()),
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
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

/// Shown right after a transfer's numbers come up - this is the actual
/// connection between Hohmann Transfer and Propulsion the spec calls
/// for: the total Δv this transfer needs gets handed straight to the
/// propellant calculator, so the user doesn't have to write it down
/// and re-enter it by hand.
///
/// The spec's example also lists a "Save result" option; there's no
/// persistence layer anywhere in this app yet (nothing writes to disk,
/// and nothing else expects a "saved results" concept to exist), so
/// rather than add a menu item that doesn't actually do anything,
/// this only offers the two choices that currently mean something.
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
    println!("║              HOHMANN TRANSFER                ║");
    println!("╚══════════════════════════════════════════════╝\n");
    println!("{}\n", body.name);

    println!("INITIAL ORBIT");
    println!("Altitude:          {:>16.6} km", initial_alt_km);
    println!("Radius:            {:>16.6} km", r1_km);
    println!("Circular velocity:  {:>16.6} km/s\n", v1_km_s);

    println!("TARGET ORBIT");
    println!("Altitude:          {:>16.6} km", target_alt_km);
    println!("Radius:            {:>16.6} km", r2_km);
    println!("Circular velocity:  {:>16.6} km/s\n", v2_km_s);

    println!("TRANSFER ORBIT");
    println!("Semi-major axis:     {:>16.6} km", at_km);

    println!("\n────────────────────────────────────────────────\n");

    println!("BURN #1");
    println!("Δv:                {:>16.6} km/s   ({:>10.3} m/s)\n", dv1_km_s, dv1_km_s * 1000.0);

    println!("BURN #2");
    println!("Δv:                {:>16.6} km/s   ({:>10.3} m/s)\n", dv2_km_s, dv2_km_s * 1000.0);

    println!("TOTAL Δv:");
    println!("                   {:>16.6} km/s   ({:>10.3} m/s)\n", total_dv_km_s, total_dv_km_s * 1000.0);

    println!("TRANSFER TIME:");
    println!("                    {:>16.6} minutes", transfer_time_min);
    println!("                    {:>16.3} seconds", transfer.transfer_time_s());

    if target_alt_km < initial_alt_km {
        println!("\nNote: target orbit is lower - both burns are retrograde");
        println!("(this lowers the orbit rather than raising it).");
    }
}
