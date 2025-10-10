/*
    (3):
    Copy can only be derived for structs whose members are all Copyable:
    when there is no underlying data that needs to be moved.
    Copy can be derived if we remove the unit field.
*/
#[derive(Debug, Clone, Default)]
struct NumberWithUnit {
    unit: String,
    value: f64,
}

impl NumberWithUnit {
    fn unitless(value: f64) -> Self {
        Self {
            unit: String::new(),
            value,
        }
    }

    fn with_unit(value: f64, unit: String) -> Self {
        Self { unit, value }
    }

    fn with_unit_from(value: f64, unit: String) -> Self {
        Self {
            unit: unit.clone(),
            value,
        }
    }
}

fn main() {
    let nwu1 = NumberWithUnit::unitless(1.0);
    let nwu2 = NumberWithUnit::with_unit(1.0, String::from("km"));
    let nwu3 = NumberWithUnit::with_unit_from(1.0, String::from("m/s^2"));

    println!("{:?}", nwu1);
    println!("{:?}", nwu2);
    println!("{:?}", nwu3);
}
