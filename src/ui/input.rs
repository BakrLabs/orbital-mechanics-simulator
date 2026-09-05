use std::io::{self, Write};

fn read_line() -> String {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(_) => buf.trim().to_string(),
        Err(_) => String::new(), // EOF etc - fall through as invalid input rather than panic
    }
}

pub fn prompt(message: &str) -> String {
    print!("{}", message);
    let _ = io::stdout().flush(); // print! alone won't show up until the buffer flushes
    read_line()
}

pub fn read_menu_choice(message: &str) -> i32 {
    prompt(message).parse::<i32>().unwrap_or(-1) // -1 isn't used by any menu, falls into the invalid branch
}

pub fn pause(message: &str) {
    print!("\n{}", message);
    let _ = io::stdout().flush();
    let _ = read_line();
}
