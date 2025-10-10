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

    fn mul_vals(slice: &[NumberWithUnit]) -> NumberWithUnit {
        let mut result = NumberWithUnit::unitless(0.0);
        for item in slice {
            result.mul_in_place(item);
        }
        result
    }

    fn mul_vals_vec(vec: Vec<NumberWithUnit>) -> NumberWithUnit {
        let mut result = NumberWithUnit::unitless(0.0);
        for item in vec {
            result.mul_in_place(&item);
        }
        result
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

    /* multiply values in a slice */
    let nwu_arr = [
        NumberWithUnit::with_unit(1.0, String::from("k")),
        NumberWithUnit::with_unit(2.0, String::from("m")),
        NumberWithUnit::with_unit(3.0, String::from("1/s")),
        NumberWithUnit::with_unit(4.0, String::from("1/s")),
    ];

    let newton1 = NumberWithUnit::mul_vals(&nwu_arr);
    let newton2 = NumberWithUnit::mul_vals(&nwu_arr);
    let newton3 = NumberWithUnit::mul_vals(&nwu_arr);
    println!("newton1 = {:?}, newton2 = {:?}, newton3 = {:?}", newton1, newton2, newton3);

    /* multiply values in a vector */
    let nwu_vec = vec![
        NumberWithUnit::with_unit(1.0, String::from("k")),
        NumberWithUnit::with_unit(2.0, String::from("m")),
        NumberWithUnit::with_unit(3.0, String::from("1/s")),
        NumberWithUnit::with_unit(4.0, String::from("1/s")),
    ];

    let newton1_vec = NumberWithUnit::mul_vals_vec(nwu_vec.clone());
    let newton2_vec = NumberWithUnit::mul_vals_vec(nwu_vec.clone());
    let newton3_vec = NumberWithUnit::mul_vals_vec(nwu_vec.clone());
    // this will not work. The first call takes ownership of the vector
    // let newton4_vec = NumberWithUnit::mul_vals_vec(nwu_vec);
    // let newton5_vec = NumberWithUnit::mul_vals_vec(nwu_vec);
    println!("newton1_vec = {:?}, newton2_vec = {:?}, newton3_vec = {:?}", newton1_vec, newton2_vec, newton3_vec);
}
