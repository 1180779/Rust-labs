use std::io;
use std::io::Read;
// use clap::Parser;
use pest::Parser;
use proj_1::{parsing, parse_query};
use proj_1::parsing::{GrammaParser, Rule};
//
// #[derive(Parser, Debug)]
// struct Args {
//     #[arg(short, long)]
//     key_type: String,
// }

fn main() {
    // let args = Args::parse();

    // let mut line = String::new();
    // let line_res = io::stdin().read_line(&mut line).unwrap();
    // line = line.trim().into();


    // TODO: change .pest to be ascii independent
    let mut line = String::from("SELECT field1,field2, field3    ,field4 FROM sample_table");
    line = String::from("CREATE table KEY keyfield FIELDS field1: bool, field2: int, field3: float, field4: StrIng");
    line = String::from("INSERT field1 = 'value1', field2 = 'value2', field3 = false INTO table");
    line = String::from("DELETE 'key1' FROM table");
    let parsed = parse_query::<String>(&line);
    match parsed {
        Ok(query) => {
            println!("{:?}", query);
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }

    /*let mut line = String::new();
    while let line_res = io::stdin().read_line(&mut line) {
        if let Err(e) = line_res {
            println!("Failed to read line: {:?}", e);
            continue;
        }
        // TODO: parsing
        todo!();
        break;
    }*/
}
