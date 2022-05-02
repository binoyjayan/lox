use error::LoxResult;
use interpreter::Interpreter;
use std::env;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::process;
use std::rc::Rc;
use std::str;

mod callable;
mod environment;
mod error;
mod expr;
mod functions_lox;
mod functions_native;
mod interpreter;
mod lox_class;
mod lox_instance;
mod object;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;

fn main() -> Result<(), LoxResult> {
    let args: Vec<String> = env::args().collect();
    let lox = Lox::new();

    if args.len() == 2 {
        lox.run_file(&args[1]).unwrap();
    } else if args.len() == 1 {
        lox.run_prompt()?;
    } else {
        println!("Usage: {} <lox-script>", args[0]);
        process::exit(1);
    }
    Ok(())
}

struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&self, path: &String) -> io::Result<()> {
        let buf = fs::read_to_string(path)?;
        match self.run(buf.as_str()) {
            Ok(_) => {}
            Err(_) => {
                // error already reported
                process::exit(65);
            }
        }
        Ok(())
    }

    pub fn run_prompt(&self) -> Result<(), LoxResult> {
        let stdin = io::stdin();
        print!(">> ");
        io::stdout().flush().unwrap();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                match self.run(&line) {
                    Ok(_) => {}
                    Err(_e) => {} // error already reported
                }
            }
            print!(">> ");
            io::stdout().flush().unwrap();
        }
        println!("\nExiting...");
        Ok(())
    }

    fn run(&self, source: &str) -> Result<(), LoxResult> {
        let mut scanner = scanner::Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = parser::Parser::new(tokens);
        let stmts = parser.parse()?;
        if parser.success() {
            let resolver = resolver::Resolver::new(&self.interpreter);
            let statements = Rc::new(stmts);
            resolver.resolve(&Rc::clone(&statements))?;
            if resolver.success() {
                return self.interpreter.interpret(&Rc::clone(&statements));
            }
        }
        Ok(())
    }
}
