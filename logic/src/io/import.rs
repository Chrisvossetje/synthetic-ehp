//! Loading saved action logs (`write_log`'s JSON output in `export`) back into
//! a list of [`Action`]s, so a session can be replayed instead of re-entered.

use std::{fs::File, io::Read};

use crate::{io::export::repo_root_path, solve::action::Action};


pub fn get_log(minimal: bool, ahss: bool) -> Result<Vec<Action>, ()> {
    let file_name = match (ahss, minimal) {
        (true, true) => "log_stable_minimal.json",
        (true, false) => "log_stable.json",
        (false, true) => "log_minimal.json",
        (false, false) => "log.json",
    };
    let mut f = File::open(repo_root_path(file_name)).map_err(|_| ())?;
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    serde_json::de::from_str(&s).map_err(|_| println!("{:?}", s))
}

