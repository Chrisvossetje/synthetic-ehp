//! Binary entry point. Wires together the data, domain, io, and solve modules.
//! `main` is a scratch harness: the currently-active path replays the EHP log
//! and prints the order table, while the alternative interactive/automated
//! routines are toggled in and out during development.
//!
//! `MAX_STEM`/`MAX_VERIFY_STEM` bound the range of validity of the Curtis data.

#[allow(unused)]
use crate::{
    data::curtis::{DATA, MODEL}, io::{
        export::export_order_table, import::get_log,
    }, routines::{automated_ahss, automated_ehp, interactive_ahss, interactive_ehp}, solve::{action::revert_log_and_remake, ehp::verify_geometric}
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
        interactive_ahss();
        interactive_ehp();

        automated_ahss(true);    

        let ehp = automated_ehp(true);

        verify_geometric(&ehp);
        export_order_table(&ehp);
    }

    // let (ahss, _) = interactive_ahss();

    // interactive_ehp();

    // automated_ahss(true);
    
    let ehp = automated_ehp(true);
    
    verify_geometric(&ehp);
    export_order_table(&ehp);
}
