use std::time::Instant;


use crate::{
    data::curtis::{DATA, MODEL, STABLE_MODEL}, io::{
        export::write_all, import::get_log,
    }, routines::{automated_ahss, automated_ehp, interactive_ahss, interactive_ehp}, solve::{automated::ahss_solver, ehp::verify_geometric}
};

mod data;
mod domain;
mod io;
mod solve;
mod types;
mod routines;

// AHSS CURTIS DATA IS VALID UNTIL STEM 48
// EHP curtis data is also valid until STEM 48
const MAX_STEM: i32 = 48;
const MAX_VERIFY_STEM: i32 = 47;



// TODO
// TODO
// TODO
// TODO
// TODO
// TODO

// STEM 36 
// AF 12

// Differential
// From: 7 3 3 6 6 5 3[4]
// To: 2 4 3 3 3 6 6 5 3[1]
// Kind: Fake
// Page: E3
// Coefficient: τ^1


// OH, i just haven't computed AHSS far enough


fn main() {
    if 1 != 1 {
        let (ahss, _) = interactive_ahss();
        interactive_ehp();
        automated_ahss(true);
        automated_ehp(true);
    }

    // let (ahss, _) = interactive_ahss();

    // interactive_ehp();

    // automated_ahss(true);
    
    let ehp = automated_ehp(true);
    verify_geometric(&ehp, &MODEL);



    // export_order_table(&ehp);

    // let (ahss, input_time_ahss) = ahss();
    // let start_ehp = Instant::now();
    // let (ehp, input_time_ehp) = ehp(&ahss);

    // verify_geometric(&ehp);

    // println!(
    //     "\nAHSS Compute took: {:.2?}",
    //     start.elapsed() - input_time_ahss - start_ehp.elapsed()
    // );
    // println!(
    //     "EHP Compute took: {:.2?}",
    //     start_ehp.elapsed() - input_time_ehp
    // );
    // println!(
    //     "Compute took: {:.2?}",
    //     start.elapsed() - input_time_ahss - input_time_ehp
    // );
    // println!("\nInput took: {:.2?}", input_time_ahss + input_time_ehp);
}
