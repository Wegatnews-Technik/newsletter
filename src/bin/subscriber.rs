use newsletter;
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    if newsletter::check_email(&input.trim().to_lowercase()) {
        println!("{input}");
    } else {
        println!("Rejected");
    }
}
