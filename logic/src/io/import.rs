use std::{fs::File, io::Read};

use crate::{io::export::repo_root_path, solve::action::Action};


pub fn get_log(minimal: bool, ahss: bool) -> Result<Vec<Action>, ()> {
    let log_path = if ahss {
        if minimal {
            repo_root_path("log_stable_minimal.json")
        } else {
            repo_root_path("log_stable.json")
        }
    } else {
        if minimal {
            repo_root_path("log_minimal.json")
        } else {
            repo_root_path("log.json")
        }
    };
    let mut f = File::open(&log_path).map_err(|_| ())?;
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    serde_json::de::from_str(&s).map_err(|_| println!("{:?}", s))
}

