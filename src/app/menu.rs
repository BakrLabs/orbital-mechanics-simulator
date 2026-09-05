use crate::app::hohmann_transfer;
use crate::app::orbital_mechanics;
use crate::app::propagation;
use crate::app::propulsion;
use crate::ui;

pub fn main_menu_loop() {
    loop {
        ui::display::clear_screen();
        ui::display::main_menu();

        match ui::input::read_menu_choice("Select an option: ") {
            1 => orbital_mechanics::menu_loop(),
            2 => hohmann_transfer::menu_loop(),
            3 => propulsion::menu_loop(),
            4 => propagation::menu_loop(),
            5 => settings_placeholder(),
            6 => about_page(),
            0 => break,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}

fn settings_placeholder() {
    ui::display::clear_screen();
    println!("SETTINGS\n");
    println!("Nothing to configure yet - this is here so the menu");
    println!("structure doesn't need to change later.\n");
    ui::input::pause("Press ENTER to return...");
}

fn about_page() {
    ui::display::clear_screen();
    println!("ABOUT\n");
    println!("Orbital Mechanics Simulator v{}", super::VERSION);
    println!("An interactive terminal tool for orbital mechanics,");
    println!("Hohmann transfers, propulsion, and numerical orbit");
    println!("propagation - in 2D and 3D, across four central bodies");
    println!("(Earth, Moon, Mars, Sun), built incrementally in Rust.\n");
    println!("See README.md and DEVLOG.md for details, including how");
    println!("the physics has been verified and where the accuracy");
    println!("limits are.\n");
    ui::input::pause("Press ENTER to return...");
}
