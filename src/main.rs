#![feature(phase)]

#[phase(plugin)]
extern crate peg_syntax_ext;
extern crate readline;

use std::cell::RefCell;
use std::rc::Rc;

mod lisp;
mod builtin;

peg_file! parser("lisp.rustpeg");

fn main() {
    println!("lisp-rs version 0.0.1");
    println!("Press Ctrl-C to Exit.\n");

    let ctx = Rc::new(RefCell::new(lisp::Context::global()));

    loop {
        match readline::readline("lisp-rs> ") {
            Some(input) => {
                readline::add_history(input.as_slice());

                let expr = parser::expr(input.as_slice());

                match expr {
                    Ok(e) => match lisp::eval(e, ctx.clone()) {
                        Ok(r) => println!("{}", r),
                        Err(r) => println!("{}", r)
                    },
                    Err(e) => println!("{}", e)
                }
            }
            None => {
                println!("\nthanks for lisping!");
                break;
            }
        }
    }
}
