#![feature(phase)]

#[phase(plugin)]
extern crate peg_syntax_ext;

use arithmetic::expression;

peg! arithmetic(r#"
#[pub]
atom -> int
    = number

number -> int
    = [0-9]+ { match_str.parse().unwrap() }

"#);

