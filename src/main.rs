#![feature(phase)]

#[phase(plugin)]
extern crate peg_syntax_ext;
extern crate readline;

mod lisp;

peg_file! parser("lisp.rustpeg");

fn main() {
    println!("lisp-rs version 0.0.1");
    println!("Press Ctrl-C to Exit.\n");

    loop {
        match readline::readline("lisp-rs> ") {
            Some(input) => {
                readline::add_history(input.as_slice());
                let expr = parser::expr(input.as_slice());
                println!("{}", lisp::eval(expr.unwrap()));
            }
            None => {
                println!("\nthanks for lisping!");
                break;
            }
        }
    }
}
