use clap::{Parser, ValueEnum};
use proj_1::{AnyCommand, AnyDatabase, Command, parse_command};
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

    println!("Empty query ends");
    loop {
        line.clear();
        let line_res = io::stdin().read_line(&mut line);
        if let Err(e) = line_res {
            println!("Failed to read line: {:?}", e);
            continue;
        }

        let line_res = line_res.unwrap();
        if line_res == 0 {
            println!("Shutting down...");
            break;
        }

        let parse_res = parse_command(&mut db, &line);
        let mut command: AnyCommand = match parse_res {
            Ok(command) => command,
            Err(e) => {
                println!("Failed to parse command: {:?}", e);
                continue;
            }
        };

        println!("Parsed command: {:?}", command);
        let result = command.execute();
        println!("Execution result: {:?}", result);
        println!("--------------------------");
        println!();
        let query_owned = command.query().into();
        db.history_push(query_owned);
    }
}
