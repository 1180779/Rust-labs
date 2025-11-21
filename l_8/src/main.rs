use std::cell::Cell;

struct AustroHungarianGreeter {
    greeting: Cell<i8>,
    greetings_invoked: Cell<u32>,
}

impl AustroHungarianGreeter {
    fn greet(&self) -> &str {
        self.greetings_invoked.set(self.greetings_invoked.get() + 1);
        match self.greeting.get() {
            0 => {
                self.greeting.set(1);
                "Es lebe der Kaiser!"
            },
            1 => {
                self.greeting.set(2);
                "Möge uns der Kaiser schützen!"
            },
            _ => {
                self.greeting.set(0);
                "Éljen Ferenc József császár!"
            }
        }
    }
}

impl Drop for AustroHungarianGreeter {
    fn drop(&mut self) {
        println!("Ich habe {} mal gegrüßt", self.greetings_invoked.get());
    }
}

fn main() {

    println!("Hello, world!");
}
