use std::{process::exit, time::Instant};

use crate::{curtis::generate_algebraic_data, export::write_typescript_file, processor::{add_diffs, add_induced_names, add_tau_mults, compute_inductive_generators}, types::{Differential, Generator, SyntheticEHP}, verification::{verify_algebraic, verify_classical, verify_integrity, verify_self_coherence, verify_stable}};

mod curtis;
mod types;
mod processor;
mod naming;
mod export;
mod verification;
mod data;

const MAX_STEM: i32 = 26;
const MAX_VERIFY_STEM: i32 = 24;
const MAX_VERIFY_SPHERE: i32 = MAX_VERIFY_STEM + 5;
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
            proof: Some("Lifted AEHP differential.".to_string()),
            synthetic: None,
        });
    }
}

fn main() {
    let start = Instant::now();

    let mut data = generate_algebraic_data();

    add_diffs(&mut data);
    add_induced_names(&mut data);
    add_tau_mults(&mut data);

    data.differentials.sort();
    
    
    compute_inductive_generators(&mut data);

    // add_final_diagonal(&mut data);
    write_typescript_file("../site/src/data.ts", &data).unwrap();
    println!("\n-----\nTesting if data is well-defined, meaning differentials / multiplications understand have generators which exist.)\n-----\n");
    if !verify_integrity(&data) {
        exit(1);
    }

    println!("\n-----\nTesting if Synthetic data is self coherent. (Rows coincide with convergence of SS)\n-----\n");
    if !verify_self_coherence(&data) {
        exit(1);
    }   

    println!("\n-----\nTesting Classical stable correctness\n-----\n");
    if !verify_stable(&data) {
        // exit(1);
    }

    println!("\n-----\nTesting Classical unstable correctness\n-----\n");
    if !verify_classical(&data) {
        // exit(1);
    }

    println!("\n-----\nTesting Algebraic correctness (Both stably and unstably)\n-----\n");
    if !verify_algebraic(&data) {
        // exit(1);
    }

    add_final_diagonal(&mut data);
    write_typescript_file("../site/src/data.ts", &data).unwrap();

    println!("\nProgram took: {:.2?}", start.elapsed());
}



#[test]
fn generate_table() {
    // Table copied from Google Sheets (tabs and newlines)
    // This contains the orders of the group 
    let table_str = "2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2
1	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2
1	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2
1	2	12	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24	24
1	12	2	4	2	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	2	2	4	2	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	2	3	72	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2	2
1	3	15	15	30	60	120	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240	240
1	15	2	2	2	48	8	16	8	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4
1	2	4	8	8	8	16	32	16	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8	8
1	4	24	2880	144	144	48	1152	48	24	12	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6	6
1	24	336	2688	2016	2016	1008	1008	1008	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504	504
1	336	4	64	8	240	1	1	1	12	2	4	2	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	4	6	288	12	6	6	12	6	6	12	12	6	3	3	3	3	3	3	3	3	3	3	3	3	3	3	3	3	3	3	3
1	6	30	30240	12	24	96	23040	64	32	32	384	32	16	8	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4
1	30	30	30	60	360	960	3840	1920	960	480	480	960	960	960	960	960	960	960	960	960	960	960	960	960	960	960	960	960	960	960	960
1	30	12	72	4	2016	16	128	16	480	2	2	2	48	8	16	8	4	4	4	4	4	4	4	4	4	4	4	4	4	4	4
1	12	48	4608	16	16	16	96	16	8	8	16	16	16	32	64	32	16	16	16	16	16	16	16	16	16	16	16	16	16	16	16
1	48	48	46080	96	288	48	24192	48	96	64	15360	128	128	128	3072	128	64	32	16	16	16	16	16	16	16	16	16	16	16	16	16
1	48	264	4224	528	8448	528	528	528	1584	2112	8448	2112	2112	1056	1056	1056	528	528	528	528	528	528	528	528	528	528	528	528	528	528	528
1	264	4	64	24	5760	24	72	24	12096	96	768	192	5760	24	24	24	288	48	96	48	24	24	24	24	24	24	24	24	24	24	24
1	4	2	32	4	2	4	32	8	8	16	32	32	16	8	16	8	8	16	16	8	4	4	4	4	4	4	4	4	4	4	4
1	2	2	32	8	32	64	4096	128	64	64	512	64	128	128	2048	128	64	64	256	32	16	8	4	4	4	4	4	4	4	4	4
1	2	4	32	32	2048	1024	8192	2048	2048	512	512	512	512	2048	8192	2048	1024	512	512	512	256	256	256	256	256	256	256	256	256	256	256
1	4	8	32	16	512	128	4096	128	1024	32	16	16	128	64	512	64	128	8	4	4	16	8	16	8	4	4	4	4	4	4	4
1	8	32	2048	128	1024	256	32768	256	64	64	128	32	32	64	128	64	16	8	4	4	4	8	16	8	4	4	4	4	4	4	4
1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1	1
";

    // Parse the string into a 2D Vec<i32>
    let table: Vec<Vec<i32>> = table_str
        .lines() // split into rows
        .map(|line| {
            line.split('\t')          // split each row into columns
                .map(|s| s.parse::<i32>().unwrap()) // parse each cell to i32
                .collect()
        })
        .collect();

    // Print the 2D table
    println!("2D table: {:?}", table);
}
