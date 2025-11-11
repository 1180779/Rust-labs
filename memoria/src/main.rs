use clap::{Parser, ValueEnum};
use memoria::{AnyDatabase, Command, parse_command};
use std::io;

#[derive(Clone, Debug, ValueEnum)]
enum KeyType {
    String,
    Int,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_enum)]
    key_type: KeyType,
}

fn main() {
    let mut db: AnyDatabase = match Args::parse().key_type {
        KeyType::String => AnyDatabase::new_string_database(),
        KeyType::Int => AnyDatabase::new_int_database(),
    };

    println!("enter QUIT to shut down");
    run(&mut db);
}

fn run(db: &mut AnyDatabase) {
    let mut line = String::new();
    loop {
        line.clear();
        if let Err(e) = io::stdin().read_line(&mut line) {
            println!("Failed to read line: {:?}", e);
            continue;
        }

        let line = line.trim();
        if line == "QUIT" {
            println!("Shutting down...");
            break;
        }
        if line.is_empty() {
            continue;
        }

        parse_and_execute(db, line);
    }
}

fn parse_and_execute(db: &mut AnyDatabase, line: &str) {
    let mut command = match parse_command(db, line) {
        Ok(cmd) => cmd,
        Err(e) => {
            println!("Failed to parse command: {:?}", e);
            return;
        }
    };

    match command.execute() {
        Ok(res) => {
            println!("{}", res);
            let query = (&command.query()).into();
            db.history_push(query);
        }
        Err(e) => println!("Failed to execute command: {}", e),
    }
}
