//! Interactive terminal menu used to enter spectral-sequence facts by hand
//! (differentials, tau-multiplications, generators, induced names) and to revert
//! previous choices. Each menu selection is translated into an [`Action`].

use std::io::{self, Write};

use crate::{
    solve::action::Action,
    types::{Kind, Torsion},
};

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read");
    input.trim().to_string()
}

fn read_int(positive: bool) -> i32 {
    loop {
        let input = read_line();
        match input.parse::<i32>() {
            Ok(v) => {
                if positive && v < 0 {
                    print!("Please enter a positive integer: ");
                    io::stdout().flush().unwrap();
                } else {
                    return v;
                }
            }
            Err(_) => {
                print!("Please enter a valid integer: ");
                io::stdout().flush().unwrap();
            }
        }
    }
}

/// Print `<label>: ` and read a trimmed line of input.
fn prompt_line(label: &str) -> String {
    print!("\n{label}: ");
    io::stdout().flush().unwrap();
    read_line()
}

/// Print `<label>: ` and read a non-negative integer.
fn prompt_int(label: &str) -> i32 {
    print!("\n{label}: ");
    io::stdout().flush().unwrap();
    read_int(true)
}

pub fn process_input(ahss: bool) -> Result<Action, ()> {
    loop {
        if ahss {
            println!("Select AHSS option:");
        } else {
            println!("Select EHP option:");
        }
        println!("1 - Add Diff");
        println!("2 - Add Internal Tau");
        println!("3 - Add External Tau");
        if ahss {
            println!("4 - Set E1 Generator");
        }
        if !ahss {
            println!("5 - Set induced name");
        }
        println!("7 - Revert Previous Choice");
        println!("8 - Revert Previous n Choices");
        if ahss {
            println!("0 - Continue to EHP");
        } else {
            println!("0 - Exit");
        }

        match prompt_int("Choice") {
            0 => {
                return Err(());
            }

            1 => {
                return Ok(Action::AddDiff {
                    from: prompt_line("From"),
                    to: prompt_line("To"),
                    proof: Some(prompt_line("Proof")),
                    kind: Kind::Real,
                });
            }

            2 => {
                return Ok(Action::AddInt {
                    from: prompt_line("From"),
                    to: prompt_line("To"),
                    page: prompt_int("Page"),
                    proof: prompt_line("Proof"),
                    kind: Kind::Real,
                });
            }

            3 => {
                return Ok(Action::AddExt {
                    from: prompt_line("From"),
                    to: prompt_line("To"),
                    af: prompt_int("Valid for AF (can insert 0 if always)"),
                    proof: Some(prompt_line("Proof")),
                    kind: Kind::Real,
                });
            }
            4 => {
                if !ahss {
                    println!("Cannot set E1 generators in EHP mode\n");
                    continue;
                }
                let tag = loop {
                    let elt = prompt_line("Name");
                    if let Some((tag, _)) = elt.split_once('[') {
                        break tag.to_string();
                    }
                    println!(
                        "Name was not a valid name. It didn't contain [, so the tag could not be deduced."
                    );
                };

                return Ok(Action::SetE1 {
                    tag,
                    torsion: Torsion::new(prompt_int("Torsion")),
                    proof: prompt_line("Proof"),
                });
            }
            5 => {
                if ahss {
                    println!("Cannot set induced names in AHSS mode");
                    continue;
                }
                return Ok(Action::SetInducedName {
                    name: prompt_line("Original"),
                    new_name: prompt_line("Induced"),
                    sphere: prompt_int("From Sphere"),
                    proof: prompt_line("Proof"),
                });
            }
            7 => {
                return Ok(Action::Revert { times: 1 });
            }
            8 => {
                return Ok(Action::Revert {
                    times: prompt_int("Times"),
                });
            }

            _ => {
                println!("Unknown option.");
            }
        }
    }
}
