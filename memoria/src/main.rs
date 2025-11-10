use clap::{Parser, ValueEnum};
use memoria::{AnyCommand, AnyDatabase, Command, parse_command};
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
    let args = Args::parse();
    let mut line = String::new();

    let mut db: AnyDatabase = match args.key_type {
        KeyType::String => AnyDatabase::new_string_database(),
        KeyType::Int => AnyDatabase::new_int_database(),
    };

    println!("enter QUIT to shut down");
    loop {
        line.clear();
        let line_res = io::stdin().read_line(&mut line);
        if let Err(e) = line_res {
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

        let parse_res = parse_command(&mut db, line);
        let mut command: AnyCommand = match parse_res {
            Ok(command) => command,
            Err(e) => {
                println!("Failed to parse command: {:?}", e);
                continue;
            }
        };

        let result = command.execute();
        match result {
            Ok(res) => {
                println!("{}", res);
                let query = (&command.query()).into();
                db.history_push(query);
            }
            Err(e) => {
                println!("Failed to execute command: {:?}", e);
                continue;
            }
        }
    }
}
