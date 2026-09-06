// ANSI clear - works on any modern terminal, and worst case on
// something exotic it just prints a blank screen instead of crashing
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

const BOX_INNER_WIDTH: usize = 50;

fn centered(text: &str) -> String {
    let pad = BOX_INNER_WIDTH.saturating_sub(text.chars().count());
    let left = pad / 2;
    let right = pad - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

pub fn title_screen(version: &str) {
    let border_top = format!("╔{}╗", "═".repeat(BOX_INNER_WIDTH));
    let border_bottom = format!("╚{}╝", "═".repeat(BOX_INNER_WIDTH));
    let blank = format!("║{}║", " ".repeat(BOX_INNER_WIDTH));

    println!("{border_top}");
    println!("{blank}");
    println!("║{}║", centered("VIS-VIVA"));
    println!("║{}║", centered("Orbital Mechanics Simulator"));
    println!("║{}║", centered(&format!("v{version}")));
    println!("{blank}");
    println!("║{}║", centered("Interactive CLI Tool"));
    println!("{blank}");
    println!("{border_bottom}");
    println!();
}

const MENU_INNER_WIDTH: usize = 48;

fn menu_line(text: &str) -> String {
    let pad = MENU_INNER_WIDTH.saturating_sub(text.chars().count());
    format!("│{}{}│", text, " ".repeat(pad))
}

fn menu_title(text: &str) -> String {
    let pad = MENU_INNER_WIDTH.saturating_sub(text.chars().count());
    let left = pad / 2;
    let right = pad - left;
    format!("│{}{}{}│", " ".repeat(left), text, " ".repeat(right))
}

pub fn main_menu() {
    let border_top = format!("┌{}┐", "─".repeat(MENU_INNER_WIDTH));
    let border_mid = format!("├{}┤", "─".repeat(MENU_INNER_WIDTH));
    let border_bottom = format!("└{}┘", "─".repeat(MENU_INNER_WIDTH));
    let blank = menu_line("");

    println!("{border_top}");
    println!("{}", menu_title("MAIN MENU"));
    println!("{border_mid}");
    println!("{blank}");
    println!("{}", menu_line("  1. Orbital Mechanics"));
    println!("{}", menu_line("  2. Hohmann Transfer"));
    println!("{}", menu_line("  3. Propulsion"));
    println!("{}", menu_line("  4. Orbit Propagation"));
    println!("{}", menu_line("  5. Settings"));
    println!("{}", menu_line("  6. About"));
    println!("{}", menu_line("  0. Exit"));
    println!("{blank}");
    println!("{border_bottom}");
    println!();
}
