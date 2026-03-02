use std::{cell::LazyCell, collections::HashMap, fs::File, io::{BufRead, BufReader}, sync::LazyLock};

use crate::{MAX_STEM, types::Torsion};

// (stem, af) -> Sorted vec of tau-modules
type SYNTHETIC_COMPARE_DATA = HashMap<(i32,i32), Vec<Torsion>>;

 static S0: LazyLock<SYNTHETIC_COMPARE_DATA> = 
    LazyLock::new(|| {
        let file_name = format!("../AHSS_DATA/S0_AdamsE2_ss.csv",);
        read_csv(1, 256, &file_name)
    });


// (bot_trunc, top_trunc) -> Compare data
pub static RP: LazyLock<HashMap<(i32,i32), SYNTHETIC_COMPARE_DATA>> = 
    LazyLock::new(|| {
        let mut m = HashMap::new();
        for i in (2..52).step_by(2) {
            // Top truncated
            m.insert((1,i), read_rp_csv(1, i));
            
            // Bot truncated
            m.insert((i-1,256), read_rp_csv(i-1, 256));
        }
        m
    });



// This bot / top trunc is for compatibility with C2, which is shifted 1 down wrt. RP1_2
// So for S0, we just dont do anything with bot trunc and toptrunc
fn read_csv(bot_trunc: i32, top_trunc: i32, file_name: &str) ->  HashMap<(i32, i32), Vec<Torsion>> {
    let mut m = HashMap::new();
    
    if let Ok(f) = File::open(file_name) {
        for l in BufReader::new(f).lines().skip(1) {
            let s = l.unwrap();
            let spl: Vec<_> = s.split(',').collect();
    
            let mut stem: i32 = spl[0].parse().unwrap();
            let mut af: i32 = spl[1].parse().unwrap();
            let dr: i32 = spl[spl.len()-1].parse().unwrap();
    
            af += 1;
    
            if bot_trunc == 1 && top_trunc == 2 {
                stem += 1;
            }
    
            if stem <= MAX_STEM {
                if dr == 9000 {
                    m.entry((stem, af)).or_insert(vec![]).push(None);
                } 
                if dr < 9000 {
                    m.entry((stem, af)).or_insert(vec![]).push(Some(dr - 1));
                }
            }
        }
    } else {
        panic!()
    }
    for j in &mut m {
        j.1.sort();
    }
    m
}

pub fn read_rp_csv(bot_trunc: i32, top_trunc: i32) -> HashMap<(i32, i32), Vec<Option<i32>>> {
    let file_name = format!("../AHSS_DATA/RP{bot_trunc}_{top_trunc}_AdamsE2_ss.csv",);
    read_csv(bot_trunc, top_trunc, &file_name)
} 