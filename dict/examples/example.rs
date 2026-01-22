use dict::RedBlackTree;

fn main() {
    println!("=== Red-Black Tree Dictionary Example (Rust) ===");

    println!("\nCreating dictionary");
    let d = RedBlackTree::new();
    let Some(mut d) = d else {
        println!("Dictionary creation failed. Exiting");
        return;
    };

    d.insert(1, "a".into());
    d.insert(12, "ab".into());
    d.insert(123, "abc".into());
    d.insert(1234, "abcd".into());
    println!("Inserted keys: 1, 12, 123, 1234");

    println!("\nChecking if keys exist:");
    let keys_to_check = [1, 12, 123, 1234, 2, 5, 56, 79, 1239];
    for key in keys_to_check {
        println!("\tContains key {:5} {}", key,  d.contains(key));
    }

    println!("\nFinding values:");
    let keys_to_find = keys_to_check;
    for key in keys_to_find {
        let result = d.find(key);
        match result {
            Some(value) => println!("\t{:10} {:5} {}", "Found", key, value),
            None => println!("\t{:10} {:5}", "Not Found", key)
        }
    }

    println!("\nGet minimum");
    let minimum = d.minimum();
    match minimum {
        Some(value) => println!("\t{:15} {}", "Minimum", value),
        None => println!("\t{:15}", "Dict empty")
    }

    println!("\nGet maximum");
    let maximum = d.maximum();
    match maximum {
        Some(value) => println!("\t{:15} {}", "Maximum", value),
        None => println!("\t{:15}", "Dict empty")
    }

    println!("\nDeleting keys");
    let keys_to_delete = keys_to_check;
    for key in keys_to_delete {
        let result = d.remove(key);
        match result {
            true => println!("\t{:15} {:5}", "Removed value", key),
            false => println!("\t{:15} {:5}", "Was not present", key)
        }
    }
}