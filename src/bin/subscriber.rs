use std::io;

#[tokio::main]
async fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    if newsletter::check_email_syntax(&input.trim().to_lowercase()) {
        println!("{input}");
        println!("{:?}", newsletter::email_exists(input.trim()).await);
    } else {
        println!("Rejected");
    }
}
