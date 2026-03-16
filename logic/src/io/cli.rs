use std::io::{self, Write};

use crate::{solve::action::Action, types::Torsion};


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
                    return v
                }
            },
            Err(_) => {
                print!("Please enter a valid integer: ");
                io::stdout().flush().unwrap();
            }
        }
    }
}

pub fn process_input(ahss: bool) -> Result<Action, ()>{
    loop {
        println!("Select option:");
        println!("0 - Exit");
        println!("1 - Add Diff");
        println!("2 - Add Internal Tau");
        println!("3 - Add External Tau");
        println!("4 - Set E1 Generator");
        println!("5 - Revert Previous Choice");
        println!("6 - Revert Previous n Choices");
    
        print!("\nChoice: ");
        io::stdout().flush().unwrap();
    
        let choice = read_int(true);
    
        match choice {
            0 => {
                return Err(());
            }
    
            1 => {
                print!("\nFrom: ");
                io::stdout().flush().unwrap();
                let from = read_line();
                print!("\nTo: ");
                io::stdout().flush().unwrap();
                let to = read_line();
                print!("\nProof: ");
                io::stdout().flush().unwrap();
                let proof = read_line();
                return Ok(Action::AddDiff { from, to, proof })
            }
            
            2 => {
                print!("\nFrom: ");
                io::stdout().flush().unwrap();
                let from = read_line();
                print!("\nTo: ");
                io::stdout().flush().unwrap();
                let to = read_line();
                print!("\nPage: ");
                io::stdout().flush().unwrap();
                let page = read_int(true);
                print!("\nProof: ");
                io::stdout().flush().unwrap();
                let proof = read_line();
                return Ok(Action::AddInt { from, to, page, proof })
            }
            
            3 => {
                print!("\nFrom: ");
                io::stdout().flush().unwrap();
                let from = read_line();
                print!("\nTo: ");
                io::stdout().flush().unwrap();
                let to = read_line();
                print!("\nProof: ");
                io::stdout().flush().unwrap();
                let proof = read_line();
                return Ok(Action::AddExt { from, to, proof })
            },
            4 => {
                let tag = loop {
                    print!("\nName: ");    
                    io::stdout().flush().unwrap();
                    let elt = read_line();
                    let name = elt.split_once('[');
                    if let Some((tag, _)) = name {
                        break tag.to_string();
                    }
                    println!("Name was not a valid name. It didn't contain [, so the tag could not be deduced.");
                };
                
                print!("\nTorsion: ");    
                io::stdout().flush().unwrap();
                let torsion = read_int(true);

                print!("\nProof: ");    
                io::stdout().flush().unwrap();
                let proof = read_line();
                
                return Ok(Action::SetE1 { tag, proof, torsion: Torsion::new(torsion) })
                
            },
            5 => {
                return Ok(Action::Revert { times: 1 });
            },
            6 => {
                print!("\nTimes: ");    
                io::stdout().flush().unwrap();
                let times = read_int(true);
                return Ok(Action::Revert { times });
            },
            
            _ => {
                println!("Unknown option.");
            }
        }
    }
}



