#![allow(non_snake_case)]

use std::rc::Rc;
//use std::collections::HashMap;
use fnv::FnvHashMap;
use itertools::Itertools;


extern crate lazy_static;
extern crate fnv;
extern crate itertools;
extern crate regex;

extern crate rustyline;

use crate::reader;

use crate::types::MalErr::{ErrMalVal, ErrString};
use crate::types::MalVal::{Bool, Func, Hash, List, MalFunc, Nil, Str, Sym, Vector};
use crate::types::{error, MalArgs, MalErr, MalRet, MalVal};

use crate::env::{env_bind, env_find_repl, env_get, env_new, env_set, Env};


fn qq_iter(elts: &MalArgs) -> MalVal {
    let mut acc = list![];
    for elt in elts.iter().rev() {
        if let List(v, _) = elt {
            if v.len() == 2 {
                if let Sym(ref s) = v[0] {
                    if s == "splice-unquote" {
                        acc = list![Sym("concat".to_string()), v[1].clone(), acc];
                        continue;
                    }
                }
            }
        }
        acc = list![Sym("cons".to_string()), quasiquote(elt), acc];
    }
    acc
}

fn quasiquote(ast: &MalVal) -> MalVal {
    match ast {
        List(v, _) => {
            if v.len() == 2 {
                if let Sym(ref s) = v[0] {
                    if s == "unquote" {
                        return v[1].clone();
                    }
                }
            }
            qq_iter(v)
        }
        Vector(v, _) => list![Sym("vec".to_string()), qq_iter(v)],
        Hash(_, _) | Sym(_) => list![Sym("quote".to_string()), ast.clone()],
        _ => ast.clone(),
    }
}

pub fn eval(orig_ast: &MalVal, orig_env: &Env) -> MalRet {
    let mut ast = orig_ast;
    let mut env = orig_env;
    // These variables ensure a sufficient lifetime for the data
    // referenced by ast and env.
    let mut live_ast;
    let mut live_env;

    'tco: loop {
        match env_get(env, "DEBUG-EVAL") {
            None | Some(Bool(false)) | Some(Nil) => (),
            _ => println!("EVAL: {}", print(ast)),
        }
        match ast {
            Sym(s) => match env_get(env, s) {
                Some(r) => return Ok(r),
                None => return error(&format!("'{}' not found", s)),
            },
            Vector(v, _) => {
                let mut lst: MalArgs = vec![];
                for a in v.iter() {
                    lst.push(eval(a, env)?);
                }
                return Ok(vector!(lst));
            }
            Hash(hm, _) => {
                let mut new_hm: FnvHashMap<String, MalVal> = FnvHashMap::default();
                for (k, v) in hm.iter() {
                    new_hm.insert(k.to_string(), eval(v, env)?);
                }
                return Ok(Hash(Rc::new(new_hm), Rc::new(Nil)));
            }
            List(l, _) => {
                if l.is_empty() {
                    return Ok(ast.clone());
                }
                let a0 = &l[0];
                match a0 {
                    Sym(a0sym) if a0sym == "def!" => {
                        return env_set(env, &l[1], eval(&l[2], env)?);
                    }
                    Sym(a0sym) if a0sym == "let*" => {
                        live_env = env_new(Some(env.clone()));
                        env = &live_env;
                        let (a1, a2) = (&l[1], &l[2]);
                        match a1 {
                            List(binds, _) | Vector(binds, _) => {
                                for (b, e) in binds.iter().tuples() {
                                    let val = eval(e, env)?;
                                    env_set(env, b, val)?;
                                }
                            }
                            _ => {
                                return error("let* with non-List bindings");
                            }
                        };
                        live_ast = a2.clone();
                        ast = &live_ast;
                        continue 'tco;
                    }
                    Sym(a0sym) if a0sym == "quote" => return Ok(l[1].clone()),
                    Sym(a0sym) if a0sym == "quasiquote" => {
                        live_ast = quasiquote(&l[1]);
                        ast = &live_ast;
                        continue 'tco;
                    }
                    Sym(a0sym) if a0sym == "defmacro!" => {
                        let (a1, a2) = (&l[1], &l[2]);
                        let r = eval(a2, env)?;
                        match r {
                            MalFunc {
                                eval,
                                ast,
                                env,
                                params,
                                ..
                            } => {
                                return env_set(
                                    &env,
                                    a1,
                                    MalFunc {
                                        eval,
                                        ast,
                                        env: env.clone(),
                                        params,
                                        is_macro: true,
                                        meta: Rc::new(Nil),
                                    },
                                )
                            }
                            _ => return error("set_macro on non-function"),
                        }
                    }
                    Sym(a0sym) if a0sym == "try*" => {
                        if l.len() < 3 {
                            live_ast = l[1].clone();
                            ast = &live_ast;
                            continue 'tco;
                        }
                        match eval(&l[1], env) {
                            Err(e) => {
                                let exc = match e {
                                    ErrMalVal(mv) => mv.clone(),
                                    ErrString(s) => Str(s.to_string()),
                                };
                                match &l[2] {
                                    List(c, _) => {
                                        live_env = env_new(Some(env.clone()));
                                        env = &live_env;
                                        env_set(env, &c[1], exc)?;
                                        live_ast = c[2].clone();
                                        ast = &live_ast;
                                        continue 'tco;
                                    }
                                    _ => return error("invalid catch block"),
                                }
                            }
                            res => return res,
                        }
                    }
                    Sym(a0sym) if a0sym == "do" => {
                        for i in 1..l.len() - 1 {
                            let _ = eval(&l[i], env)?;
                        }
                        live_ast = l.last().unwrap_or(&Nil).clone();
                        ast = &live_ast;
                        continue 'tco;
                    }
                    Sym(a0sym) if a0sym == "if" => {
                        let cond = eval(&l[1], env)?;
                        match cond {
                            Bool(false) | Nil if l.len() >= 4 => {
                                live_ast = l[3].clone();
                                ast = &live_ast;
                                continue 'tco;
                            }
                            Bool(false) | Nil => return Ok(Nil),
                            _ if l.len() >= 3 => {
                                live_ast = l[2].clone();
                                ast = &live_ast;
                                continue 'tco;
                            }
                            _ => return Ok(Nil),
                        }
                    }
                    Sym(a0sym) if a0sym == "fn*" => {
                        let (a1, a2) = (l[1].clone(), l[2].clone());
                        return Ok(MalFunc {
                            eval,
                            ast: Rc::new(a2),
                            env: env.clone(),
                            params: Rc::new(a1),
                            is_macro: false,
                            meta: Rc::new(Nil),
                        });
                    }
                    Sym(a0sym) if a0sym == "eval" => {
                        //  Hard to implement without global variables.
                        //  Normal argument evaluation.
                        live_ast = eval(&l[1], env)?;
                        ast = &live_ast;
                        live_env = env_find_repl(env);
                        env = &live_env;
                        continue 'tco;
                    }
                    _ => match eval(a0, env) {
                        Ok(f @ MalFunc { is_macro: true, .. }) => {
                            let new_ast = f.apply(l[1..].to_vec())?;
                            live_ast = new_ast;
                            ast = &live_ast;
                            continue 'tco;
                        }
                        Ok(f @ Func(_, _)) => {
                            let mut args: MalArgs = vec![];
                            for i in 1..l.len() {
                                args.push(eval(&l[i], env)?);
                            }
                            return f.apply(args);
                        }
                        Ok(MalFunc {
                            ast: mast,
                            env: menv,
                            params: mparams,
                            ..
                        }) => {
                            let mut args: MalArgs = vec![];
                            for i in 1..l.len() {
                                args.push(eval(&l[i], env)?);
                            }
                            live_env = env_bind(Some(menv.clone()), &mparams, args.to_vec())?;
                            env = &live_env;
                            live_ast = (*mast).clone();
                            ast = &live_ast;
                            continue 'tco;
                        }
                        Ok(_) => return error("attempt to call non-function"),
                        e @ Err(_) => return e,
                    },
                }
            }
            _ => return Ok(ast.clone()),
        };
    } // end 'tco loop
}

pub fn read(str: &str) -> MalRet {
    reader::read_str(str)
}

pub fn print(ast: &MalVal) -> String {
    ast.pr_str(true)
}

pub fn rep(str: &str, env: &Env) -> Result<String, MalErr> {
    let ast = read(str)?;
    let exp = eval(&ast, env)?;
    Ok(print(&exp))
}

pub fn re(str: &str, env: &Env) {
    if let Ok(ast) = read(str) {
        if eval(&ast, env).is_ok() {
            return;
        }
    }
    panic!("Error during startup");
}