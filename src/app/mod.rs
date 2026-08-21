mod hohmann_transfer;
mod menu;
mod orbital_mechanics;
mod propulsion;

use crate::ui;

pub const VERSION: &str = "0.5.0";

/// Kicks off the whole program: title screen, then hand control
/// over to the main menu loop until the user exits.
pub fn run() {
    ui::display::clear_screen();
    ui::display::title_screen(VERSION);
    ui::input::pause("Press ENTER to continue...");

    menu::main_menu_loop();

    ui::display::clear_screen();
    println!("Goodbye.\n");
}
