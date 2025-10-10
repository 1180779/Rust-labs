/*
    (3):
    Copy can only be derived for structs whose members are all Copyable:
    when there is no underlying data that needs to be moved.
    Copy can be derived if we remove the unit field.
*/
use std::ops::{Add, AddAssign};

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

    fn add(self, other: Self) -> Self {
        if self.unit != other.unit {
            panic!("Units are not the same");
        }
        Self {
            unit: self.unit,
            value: self.value + other.value,
        }
    }

    fn mul(self, other: Self) -> Self {
        Self {
            unit: self.unit + "*" + other.unit.as_str(),
            value: self.value * other.value,
        }
    }

    fn div(self, other: Self) -> Self {
        Self {
            unit: self.unit + "/" + other.unit.as_str(),
            value: self.value / other.value,
        }
    }

    fn add_in_place(&mut self, other: &Self) {
        if self.unit != other.unit {
            panic!("Units are not the same");
        }
        self.value += other.value;
    }

    fn mul_in_place(&mut self, other: &Self) {
        self.unit.push_str("*");
        self.unit.push_str(other.unit.as_str());
        self.value *= other.value;
    }

    fn div_in_place(&mut self, other: &Self) {
        self.unit.push_str("/");
        self.unit.push_str(other.unit.as_str());
        self.value /= other.value;
    }
}

fn main() {
    /* use constructors and print values */
    let nwu1 = NumberWithUnit::unitless(1.0);
    let nwu2 = NumberWithUnit::with_unit(15.0, String::from("km"));
    let nwu3 = NumberWithUnit::with_unit_from(5.0, String::from("s"));

    println!("{:?}", nwu1);
    println!("{:?}", nwu2);
    println!("{:?}", nwu3);


    /* use the functions and print values */
    let dist1 = NumberWithUnit::with_unit(530.0, String::from("m"));
    let dist2 = NumberWithUnit::with_unit(120.0, String::from("m"));
    let add_dist = dist1.add(dist2);
    println!("{:?}", add_dist);

    let vel1 = NumberWithUnit::with_unit(10.0, String::from("km"));
    let vel2 = NumberWithUnit::with_unit(10.0 / 3600.0, String::from("h"));
    let div_vel = vel1.div(vel2);
    println!("{:?}", div_vel);

    let enegry1 = NumberWithUnit::with_unit(0.8, String::from("kW"));
    let energy2 = NumberWithUnit::with_unit(8.0, String::from("h"));
    let mul_energy = enegry1.mul(energy2);
    println!("{:?}", mul_energy);

    /* use functions in place */
    /* TODO: implement */


}
