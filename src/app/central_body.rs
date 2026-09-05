use crate::physics::body::CelestialBody;
use crate::ui;

// shared across every flow that needs a central body
pub fn select() -> Option<CelestialBody> {
    loop {
        ui::display::clear_screen();
        println!("CENTRAL BODY\n");
        println!("1. Earth");
        println!("2. Moon");
        println!("3. Mars");
        println!("4. Sun");
        println!("0. Back\n");

        match ui::input::read_menu_choice("Select: ") {
            1 => return Some(CelestialBody::earth()),
            2 => return Some(CelestialBody::moon()),
            3 => return Some(CelestialBody::mars()),
            4 => return Some(CelestialBody::sun()),
            0 => return None,
            _ => {
                println!("\nNot a valid option.");
                ui::input::pause("Press ENTER to try again...");
            }
        }
    }
}
