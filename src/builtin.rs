use std::cell::RefCell;
use std::rc::Rc;

use lisp;
use lisp::{Context, Expr, Atom, Func, Error};

fn arith(l: Expr, r: Expr, op: &mut |int, int| -> Result<int, Error>) -> Result<Expr, Error> {
    match (l, r) {
        (Expr::Atom(Atom::Int(i1)), Expr::Atom(Atom::Int(i2))) => {
            Ok(Expr::Atom(Atom::Int(try!((*op)(i1, i2)))))
        },
        (Expr::Atom(Atom::Int(_)), nan) => Err(Error::InvalidType(format!("`{}` is not a number", nan))),
        (nan, Expr::Atom(Atom::Int(_))) => Err(Error::InvalidType(format!("`{}` is not a number", nan))),
        (nan1, _) => Err(Error::InvalidType(format!("`{}` is not a number", nan1)))
    }
}

fn do_arith(args: Vec<Expr>, op: &mut |int, int| -> Result<int, Error>) -> Result<Expr, Error> {
    match args.as_slice() {
        [] => Ok(Expr::Atom(Atom::Int(0))),

        [ref l] => {
            arith(Expr::Atom(Atom::Int(0)), l.clone(), op)
        },

        [ref x, xs..] => {
            xs.iter().fold(Ok(x.clone()), |m, r| {
                match m {
                    Ok(l) => arith(l, r.clone(), op),
                    Err(e) => Err(e)
                }
            })
        },
    }
}

pub fn add(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    do_arith(args, &mut |i1: int, i2: int| {
        Ok(i1 + i2)
    })
}

pub fn sub(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    do_arith(args, &mut |i1: int, i2: int| {
        Ok(i1 - i2)
    })
}

pub fn mul(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    do_arith(args, &mut |i1: int, i2: int| {
        Ok(i1 * i2)
    })
}

pub fn div(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    do_arith(args, &mut |i1: int, i2: int| {
        match i2 {
            0 => Err(Error::ZeroDivision),
            _ => Ok(i1 / i2)
        }
    })
}

pub fn list(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    Ok(Expr::Sexpr(args))
}

pub fn tail(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match args.as_slice() {
        [Expr::Sexpr(ref list)] => {
            match list.as_slice() {
                [] => Err(Error::Runtime("can't tail empty list".to_string())),
                _ => {
                    let tail = list.tail();
                    Ok(Expr::Sexpr(tail.to_vec()))
                }
            }
        },

        [ref other] => Err(Error::InvalidType(format!("`{}` not a list", other))),

        _ => Err(Error::Arity("tail expects one argument (a list)".to_string()))
    }
}

pub fn head(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match args.as_slice() {
        [Expr::Sexpr(ref list)] => {
            match list.first() {
                Some(h) => Ok(h.clone()),
                None => Err(Error::Runtime("can't head empty list".to_string())),
            }
        },

        [ref other] => Err(Error::InvalidType(format!("`{}` not a list", other))),

        _ => Err(Error::Arity("head expects one argument (a list)".to_string()))
    }
}

pub fn eval(args: Vec<Expr>, ctx: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match args.as_slice() {
        [ref a] => lisp::eval(a.clone(), ctx),

        _ => Err(Error::Arity("eval expects one argument (an sexpr)".to_string()))
    }
}

fn as_name(e: &Expr) -> Result<String, Error> {
    match *e {
        Expr::Atom(Atom::Sym(ref name)) => Ok(name.clone()),
        _ => Err(Error::InvalidType(format!("formals declaration expects a list of symbols, `{}` is not a symbol", e)))
    }
}

fn as_formals(fs: &Vec<Expr>) -> Result<Vec<String>, Error> {
    fs.iter().fold(Ok(Vec::new()), |m, a| {
        match m {
            Ok(mut xs) => {
                let name = try!(as_name(a));
                xs.push(name);
                Ok(xs)
            },

            Err(e) => Err(e)
        }
    })
}

pub fn lambda(args: Vec<Expr>, _: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match args.as_slice() {
        [Expr::Sexpr(ref fs), ref body] => {
            let formals = try!(as_formals(fs));

            Ok(Expr::Atom(Atom::Fun(Func::Lambda(formals, box body.clone()))))
        },

        _ => Err(Error::Arity("lambda expects two arguments (qexpr, qexpr)".to_string()))
    }
}

pub fn def(args: Vec<Expr>, ctx: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match args.as_slice() {
        [Expr::Atom(Atom::Sym(ref name)), ref val] => {
            ctx.borrow_mut().table.insert(name.to_string(), val.clone());
            Ok(val.clone())
        },

        _ => Err(Error::Arity("def expects two arguments (sym, expr)".to_string()))
    }
}
