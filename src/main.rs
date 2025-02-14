#![allow(non_snake_case)]

use std::rc::Rc;


#[macro_use]
extern crate lazy_static;
extern crate fnv;
extern crate itertools;
extern crate regex;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[macro_use]
mod types;
use crate::types::MalVal::{List, Nil, Str};
use crate::types::format_error;
mod env;
mod printer;
mod reader;
use crate::env::{env_new, env_sets};
#[macro_use]
mod core;
mod rep;
use crate::rep::{rep, re};

fn main() {
    let mut args = std::env::args();
    let arg1 = args.nth(1);

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
    if rl.load_history(".mal-history").is_err() {
        eprintln!("No previous history");
    }

    let repl_env = env_new(None);
    for (k, v) in core::ns() {
        env_sets(&repl_env, k, v);
    }
    env_sets(&repl_env, "*ARGV*", list!(args.map(Str).collect()));

    re("(def! *host-language* \"rust\")", &repl_env);
    re("(def! not (fn* (a) (if a false true)))", &repl_env);
    re(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))",
        &repl_env,
    );
    re("(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw \"odd number of forms to cond\")) (cons 'cond (rest (rest xs)))))))",
        &repl_env);

    if let Some(f) = arg1 {
        re(&format!("(load-file \"{}\")", f), &repl_env);
        std::process::exit(0);
    }

    re("(println (str \"Mal [\" *host-language* \"]\"))", &repl_env);

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                rl.save_history(".mal-history").unwrap();
                if !line.is_empty() {
                    match rep(&line, &repl_env) {
                        Ok(out) => println!("{}", out),
                        Err(e) => println!("Error: {}", format_error(e)),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }
    }
}
