use clap::{Parser, ValueEnum};
use proj_1::{AnyDatabase, AnyQuery, parse_query};
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

        let query: AnyQuery = match args.key_type {
            KeyType::String => {
                let parse_res = parse_query::<String>(&line);
                if parse_res.is_err() {
                    let e = parse_res.err().unwrap();
                    println!("Failed to parse query: {:?}", e);
                    continue;
                }
                AnyQuery::StringQuery(parse_res.unwrap())
            }
            KeyType::Int => {
                let parse_res = parse_query::<i64>(&line);
                if parse_res.is_err() {
                    let e = parse_res.err().unwrap();
                    println!("Failed to parse query: {:?}", e);
                    continue;
                }
                AnyQuery::IntQuery(parse_res.unwrap())
            }
        };
        println!("Parsed query: {:?}", query);
        let result = db.execute(query);
        println!("Execution result: {:?}", result);
        println!("--------------------------");
        println!();
    }
}
