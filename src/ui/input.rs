use std::io::{self, Write};

/// Reads a single line from stdin and trims it. Returns an empty
/// string on EOF instead of panicking - if someone pipes input in
/// and it runs dry, we'd rather fall through to "invalid input"
/// than blow up.
fn read_line() -> String {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(_) => buf.trim().to_string(),
        Err(_) => String::new(),
    }
}

/// Prints a prompt on the same line (flushing stdout first, since
/// print! alone won't show up until the buffer flushes) and reads
/// back whatever the user typed.
pub fn prompt(message: &str) -> String {
    print!("{}", message);
    let _ = io::stdout().flush();
    read_line()
}

/// Reads a menu choice as an integer. Anything that doesn't parse
/// cleanly - letters, blank input, decimals - comes back as -1,
/// which no menu uses, so it just falls into the "invalid" branch
/// of whatever match statement is calling this.
pub fn read_menu_choice(message: &str) -> i32 {
    prompt(message).parse::<i32>().unwrap_or(-1)
}

/// Waits for the user to hit ENTER before moving on. Used after
/// placeholder pages and results screens so things don't just fly
/// by.
pub fn pause(message: &str) {
    print!("\n{}", message);
    let _ = io::stdout().flush();
    let _ = read_line();
}
