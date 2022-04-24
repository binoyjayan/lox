use std::io;
use std::fs;
use std::env;
use std::str;
use std::process;
use std::io::{Write, BufRead};

mod expr;
mod stmt;
mod error;
mod token;
mod object;
mod parser;
mod scanner;
mod environment;
mod interpreter;
use error::LoxErr;
use interpreter::Interpreter;

fn main() -> Result<(), LoxErr> {
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
        Lox { interpreter: Interpreter::new() }
    }

    pub fn run_file(&self, path: &String) -> io::Result<()> {
        let buf= fs::read_to_string(path)?;
        match self.run(buf.as_str()) {
            Ok(_) => {},
            Err(_) => {    
                // error already reported
                process::exit(65);
            }
        }
        Ok(())
    }
    
    pub fn run_prompt(&self) -> Result<(), LoxErr> {
        let stdin = io::stdin();
        print!(">> ");
        io::stdout().flush().unwrap();
        for line in stdin.lock().lines() {        
            if let Ok(line) = line {
                match self.run(&line) {
                    Ok(_) => {},
                    Err(_e) => {} // error already reported
                }
            }
            print!(">> ");
            io::stdout().flush().unwrap();
        }
        println!("\nExiting...");
        Ok(())
    }
    
    fn run(&self, source: &str) -> Result<(), LoxErr> {
        let mut scanner = scanner::Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = parser::Parser::new(tokens);
        let stmts = parser.parse()?;
        if parser.success() {
            self.interpreter.interpret(&stmts)
        } else {
            Err(LoxErr::error(0, 0, ""))
        }
    }
}




