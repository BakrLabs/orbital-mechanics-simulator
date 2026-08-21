/// Clears the terminal. Works on basically anything ANSI-aware,
/// which covers Linux/macOS terminals and modern Windows terminals.
/// If it doesn't work in some exotic terminal, worst case we just
/// print a couple of blank lines - not worth pulling in a crate for.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn title_screen(version: &str) {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║                                                  ║");
    println!("║          ORBITAL MECHANICS SIMULATOR             ║");
    println!("║                       v{}                     ║", version);
    println!("║                                                  ║");
    println!("║              Interactive CLI Tool                ║");
    println!("║                                                  ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
}

pub fn main_menu() {
    println!("┌──────────────────────────────────────────────┐");
    println!("│                  MAIN MENU                   │");
    println!("├──────────────────────────────────────────────┤");
    println!("│                                              │");
    println!("│  1. Orbital Mechanics                        │");
    println!("│  2. Hohmann Transfer                         │");
    println!("│  3. Propulsion                               │");
    println!("│  4. Settings                                 │");
    println!("│  5. About                                    │");
    println!("│  0. Exit                                     │");
    println!("│                                              │");
    println!("└──────────────────────────────────────────────┘");
    println!();
}
