use std::time::Instant;


use crate::{
    io::{
        export::write_all, import::get_log,
    }, routines::{automated_ahss, automated_ehp, interactive_ahss, interactive_ehp}, solve::automated::ahss_solver
};

mod data;
mod domain;
mod io;
mod solve;
mod types;
mod routines;

const MAX_STEM: i32 = 48;
// TODO: AHSS CURTIS DATA IS VALID UNTIL STEM 48
// TODO: It seems EHP curtis data is also valid until +- STEM 48
const MAX_VERIFY_STEM: i32 = 48;


// pub fn add_final_diagonal(data: &mut OldSyntheticSS) {
//     // Generate the degree zero parts
//     for n in (3..MAX_UNEVEN_INPUT).step_by(4) {
//         let y = n / 2;

//         data.generators.push(Generator::new(format!("2(∞)[{}]", y), y, y, 2, 0, None));
//         data.generators.push(Generator::new(format!("1(∞)[{}]", y + 1), y + 1, y + 1, 1, 0, None));

//         data.differentials.push(Differential {
//             from: format!("1(∞)[{}]", y + 1),
//             to: format!("2(∞)[{}]", y),
//             coeff: 0,
//             d: 1,
//             proof: Some("Lifted AEHP differential.".to_string()),
//             synthetic: None,
//             kind: Kind::Real,
//         });
//     }
// }


// pub static STABLE_SYNTHETIC_PAGES: OnceLock<[SSPages; (MAX_STEM + 1) as usize]> = OnceLock::new();

// fn main() {
//     let start = Instant::now();

//     let mut log = match get_log(false, true) {
//         Ok(log) => log,
//         Err(_) => {
//             panic!("Log importing was not succesful");
//         }
//     };
    
//     let ahss = revert_log_and_remake(0, &mut log, &STABLE_DATA, true);
    
//     let ahss_pages = std::array::from_fn(|x| compute_pages(&ahss, 0, x as i32, 0, 150, false).0);
//     STABLE_SYNTHETIC_PAGES.set(ahss_pages).unwrap();
    
//     let alg_ehp_pages = std::array::from_fn(|x| compute_pages(&DATA, 0, x as i32 - 1, 0, MAX_STEM + 5, false).0);
//     ALGEBRAIC_SPHERE_PAGES.set(alg_ehp_pages).unwrap();
    
//     // if let Ok((ahss_log, ahss)) = ahss_solver(Some(log)) {
//     //     write_all(&ahss, &ahss_log, true);
    
//     // }

//     let mut ehp_log = match get_log(true, false) {
//         Ok(log) => log,
//         Err(_) => {
//             panic!("Log importing was not succesful");
//         }
//     };
        
//     let (ehp_log, ehp) = ehp_solver(&ahss, Some(ehp_log));
//     write_all(&ehp, &ehp_log, false);


//     // export_order_table(&ehp);

//     // let (ahss, input_time_ahss) = ahss();
//     // let start_ehp = Instant::now();
//     // let (ehp, input_time_ehp) = ehp(&ahss);

//     // verify_geometric(&ehp);

//     // println!(
//     //     "\nAHSS Compute took: {:.2?}",
//     //     start.elapsed() - input_time_ahss - start_ehp.elapsed()
//     // );
//     // println!(
//     //     "EHP Compute took: {:.2?}",
//     //     start_ehp.elapsed() - input_time_ehp
//     // );
//     // println!(
//     //     "Compute took: {:.2?}",
//     //     start.elapsed() - input_time_ahss - input_time_ehp
//     // );
//     // println!("\nInput took: {:.2?}", input_time_ahss + input_time_ehp);
//     println!("Program took: {:.2?}", start.elapsed());
// }

fn main() {
    if 1 != 1 {
        let (ahss, _) = interactive_ahss();
        interactive_ehp(&ahss);
        automated_ahss();
        automated_ehp(&ahss);
    }

    let start = Instant::now();

    let ahss_log = match get_log(true, true) {
        Ok(log) => Some(log),
        Err(_) => None,
    };

    let (ahss_log, ahss) = ahss_solver(ahss_log);
    // if let Ok((ahss_log, ahss)) = crate::solve::automated::ahss_solver(log) {
    //     write_all(&ahss, &ahss_log, true);
    //     Some(ahss)
    // } else {
    //     None
    // }
    
        
    // let (ehp_log, ehp) = ehp_solver(&ahss, Some(ehp_log));
    write_all(&ahss, &ahss_log, true);


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
    println!("Program took: {:.2?}", start.elapsed());
}


#[test]
fn lol1() {

}