use std::env;
fn main() {
    let cli = env::var("CLI").is_ok();
    if cli {
        println!("running in cli");
        redstone_rust::run();
    } else{
        println!("initializing");
    }
}