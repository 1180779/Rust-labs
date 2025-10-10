use rand::Rng;
use std::fs::File;
use std::io;
use std::io::Write;

fn main() {
    let mut input: String;
    let mut file = File::create("xyz.txt").expect("File not created!");

    let exited_on_error = loop {
        println!("Podaj liczbę: ");

        input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let number: u64 = match input.trim().parse::<u64>() {
            Ok(num) => num,
            Err(_) => break true,
        };
        if number == 0 {
            break false;
        }

        let x = number + rand::rng().random_range(0..=5);
        println!("x = {}", x);

        let arr = exponents(x);
        println!("arr = {:?}", arr);

        let mut colatz_arr = [false; 10];
        for i in 0..arr.len() {
            colatz_arr[i] = collatz_hypothesis(arr[i], 100);
        }
        println!("colatz_arr = {:?}", colatz_arr);

        // write all to file
        let wres = file.write_all(
            format!(
                "{:?}\n",
                colatz_arr
            )
                .as_bytes(),
        );
        if wres.is_err() {
            break true;
        }
    };

    if exited_on_error {
        println!("Exited loop on error");
    } else {
        println!("Exited loop on user request");
    }

    // my own functionality that returns a tuple with different types directly from the inner loop
    println!("\n\n--------------------------------------");
    println!("Custom functionality.");
    let tuple: (String, i32) = 'outer: loop {
        println!("\nSelect one of the modes:");
        println!("[acc] - accumulator for numbers");
        println!("[wrd] - word builder");
        println!("Enter your choice: ");
        let mut mode_input = String::new();
        io::stdin()
            .read_line(&mut mode_input)
            .expect("Failed to read line");
        mode_input = mode_input.trim().to_string();

        match mode_input.as_str() {
            "acc" => {
                println!("Selected Accumulator. Input numbers to sum. 0 ends.");
                let mut sum = 0;
                loop {
                    input.clear();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");
                    let number = input.trim().parse::<i32>();
                    match number {
                        Ok(num) => match num {
                            0 => break,
                            _ => sum += num,
                        },
                        Err(_) => {
                            println!("Not a valid number");
                        }
                    }
                }
                println!("Sum: {}", sum);
                if sum == 7 {
                    break 'outer (String::from("Sum is secret number"), 7);
                }
            }
            "wrd" => {
                println!("Selected Word builder. Input words. Empty line ends.");
                let mut word = String::new();
                loop {
                    input.clear();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");
                    let input = input.trim();
                    if input.is_empty() {
                        break;
                    }
                    word.push_str(input);
                }
                println!("Word: {}", word);
                if word == "UFO" {
                    break 'outer (String::from("UFO appeared on the sky"), 1974);
                }
            }
            _ => {
                println!("invalid mode selected. Try again.");
                continue;
            }
        };
    };

    println!(
        "Secret is: '{}', Secret code is: {}",
        tuple.0, tuple.1
    );
}

fn exponents(x: u64) -> [u64; 10] {
    let mut arr = [0; 10];
    let mut c = x;
    for item in arr.iter_mut() {
        *item = c;
        c *= x;
    }
    arr
}

fn collatz_hypothesis(mut number: u64, iter_limit: u64) -> bool {
    for _ in 0..iter_limit {
        if number == 1 {
            return true;
        }
        number = match number % 2 {
            1 => 3 * number + 1,
            0 => number / 2,
            _ => panic!(),
        }
    }
    false
}
