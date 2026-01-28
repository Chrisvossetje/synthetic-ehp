use std::process::exit;

use crate::{curtis::generate_algebraic_data, export::write_typescript_file, processor::{add_diffs, add_induced_names, compute_inductive_generators}, types::{Differential, Generator, SyntheticEHP}, verification::{verify_algebraic, verify_integrity, verify_self_coherence, verify_stable}};

mod curtis;
mod types;
mod processor;
mod naming;
mod export;
mod verification;
mod data;

const MAX_STEM: i32 = 21; // This is preferably always even
const MAX_UNEVEN_INPUT: i32 = (MAX_STEM + 1) * 2;


pub fn add_final_diagonal(data: &mut SyntheticEHP) {
        // Generate the degree zero parts
    for n in (3..MAX_UNEVEN_INPUT).step_by(4) {
        let y = n / 2;
        
        data.generators.push(Generator::new(format!("2(∞)[{}]", y), y, y, 2));
        data.generators.push(Generator::new(format!("1(∞)[{}]", y + 1), y + 1, y + 1, 1));

        data.differentials.push(Differential {
            from: format!("1(∞)[{}]", y + 1),
            to: format!("2(∞)[{}]", y),
            coeff: 0,
            d: 1,
            proof: None,
        });
    }
}

fn main() {
    let mut data = generate_algebraic_data();

    add_diffs(&mut data);
    add_induced_names(&mut data);

    compute_inductive_generators(&mut data);

    // add_final_diagonal(&mut data);
    write_typescript_file("../site/src/data.ts", &data).unwrap();
    if  !verify_integrity(&data) {
        exit(1);
    }
    if !verify_self_coherence(&data) {
        exit(1);
    }
    if !verify_algebraic(&data) {
        exit(1);
    }
    if !verify_stable(&data) {
        exit(1);
    }
}
