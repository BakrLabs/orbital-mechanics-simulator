use crate::app::hohmann_transfer;
use crate::app::orbital_mechanics;
use crate::app::propulsion;
use crate::ui;

/// Top-level menu. Orbital Mechanics, Hohmann Transfer, and Propulsion
/// all have their own modules now that they do something - Settings
/// and About are still simple enough to live inline.
pub fn main_menu_loop() {
    loop {
        ui::display::clear_screen();
        ui::display::main_menu();

        match ui::input::read_menu_choice("Select an option: ") {
            1 => orbital_mechanics::menu_loop(),
            2 => hohmann_transfer::menu_loop(),
            3 => propulsion::menu_loop(),
            4 => settings_placeholder(),
            5 => about_page(),
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
    println!("An interactive terminal tool for orbital mechanics and");
    println!("mission design, built incrementally in Rust.\n");
    ui::input::pause("Press ENTER to return...");
}
