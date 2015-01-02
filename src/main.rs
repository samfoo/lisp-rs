#![feature(phase)]

extern crate readline;

#[phase(plugin)] extern crate peg_syntax_ext;

peg_file! parser("lisp.rustpeg");

fn main() {
    println!("lisp-rs version 0.0.1");
    println!("Press Ctrl-C to Exit.\n");

    loop {
        match readline::readline("lisp-rs> ") {
            Some(input) => {
                readline::add_history(input.as_slice());
                let expr = parser::number(input.as_slice());
                println!("{}", expr);
            }
            None => {
                println!("\nthanks for lisping!");
                break;
            }
        }
    }
}
