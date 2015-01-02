extern crate readline;

fn main() {
    println!("lisp-rs version 0.0.1");
    println!("Press Ctrl-C to Exit.\n");

    loop {
        match readline::readline("lisp-rs> ") {
            Some(input) => {
                readline::add_history(input.as_slice());
                println!("echo {}", input);
            }
            None => {
                println!("\nthanks for lisping!");
                break;
            }
        }
    }
}
