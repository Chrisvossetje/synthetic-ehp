use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use itertools::{Itertools, chain};

use crate::{
    MAX_STEM, MAX_VERIFY_STEM,
    data::curtis::{DATA, STABLE_DATA},
    domain::process::compute_pages,
    solve::action::D_R_REPEATS,
    types::Torsion,
};

// (stem, af) -> Sorted vec of tau-modules
#[allow(non_camel_case_types)]
type SYNTHETIC_COMPARE_DATA = HashMap<(i32, i32), Vec<Torsion>>;

// (stem, af) -> Amount of F2 generators
#[allow(non_camel_case_types)]
type ALGEBRAIC_COMPARE_DATA = HashMap<(i32, i32), usize>;

pub static RP_TRUNCATIONS: LazyLock<Vec<(i32, i32)>> = LazyLock::new(|| {
    chain![
        vec![(1, 2), (2, 4), (3, 5), (3, 6)],
        (2..=MAX_STEM).step_by(2).map(|x| (1, x)),
        (3..=MAX_STEM)
            .rev()
            .filter(|x| x % 2 == 1)
            .map(|x| (x, 256))
    ]
    .collect()
});

pub static ALGEBRAIC_RP_TRUNCATIONS: LazyLock<Vec<(i32, i32)>> = LazyLock::new(|| {
    chain![
        vec![(1, 3), (4, 6), (2, 5), (4, 7)],
        (4..=MAX_STEM).flat_map(|l| {
            (0..(D_R_REPEATS[(l - 1) as usize] as i32).min(50 - l + 1)).map(move |i| (1 + i, l + i))
        })
    ]
    .collect()
});

pub static TRUNCS: LazyLock<Vec<(bool, i32, i32)>> = LazyLock::new(|| {
    chain![
        [
            (true, 1, 2),
            (true, 2, 4),
            (true, 3, 5),
            (true, 1, 4),
            (true, 3, 6)
        ],
        [(false, 1, 3), (false, 4, 6), (false, 2, 5), (false, 4, 7)],
        (4..=MAX_STEM)
            .flat_map(|l| {
                (0..(D_R_REPEATS[(l - 1) as usize] as i32).min(50 - l + 1)).map(move |i| {
                    (
                        synthetic_rp_truncations().contains(&(1 + i, l + i)),
                        1 + i,
                        l + i,
                    )
                })
            })
            .sorted_by_key(|x| (!x.0, x.2 - x.1)),
        (3..=MAX_STEM)
            .rev()
            .filter(|x| x % 2 == 1)
            .map(|x| (true, x, 256)),
    ]
    .collect()
});

pub static EHP_TO_AHSS: LazyLock<Vec<Option<usize>>> = LazyLock::new(|| {
    DATA
            .model
            .gens()
            .iter()
            .map(|g| STABLE_DATA.model.try_index(&g.name))
            .collect()
});

pub static AHSS_TO_EHP: LazyLock<Vec<Option<usize>>> = LazyLock::new(|| {
    STABLE_DATA
            .model
            .gens()
            .iter()
            .map(|g| DATA.model.try_index(&g.name))
            .collect()
});

pub fn synthetic_rp_truncations() -> &'static [(i32, i32)] {
    RP_TRUNCATIONS.as_slice()
}

pub fn algebraic_rp_truncations() -> &'static [(i32, i32)] {
    ALGEBRAIC_RP_TRUNCATIONS.as_slice()
}

pub fn rp_truncations() -> &'static [(bool, i32, i32)] {
    TRUNCS.as_slice()
}

pub static EMPTY_LIST_TORSION: LazyLock<Vec<Torsion>> = LazyLock::new(|| vec![]);
pub static EMPTY_LIST_USIZE: LazyLock<Vec<usize>> = LazyLock::new(|| vec![]);

pub static S0_ZEROES: LazyLock<SYNTHETIC_COMPARE_DATA> = LazyLock::new(|| {
    let file_name = ahss_data_path("S0_AdamsE2_ss.csv");
    read_csv(1, 256, &file_name, false, true)
});

pub static S0: LazyLock<SYNTHETIC_COMPARE_DATA> = LazyLock::new(|| {
    let file_name = ahss_data_path("S0_AdamsE2_ss.csv");
    read_csv(1, 256, &file_name, false, false)
});

// (bot_trunc, top_trunc) -> Compare data
pub static RP: LazyLock<HashMap<(i32, i32), SYNTHETIC_COMPARE_DATA>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for &(b, t) in synthetic_rp_truncations() {
        // Top truncated
        m.insert((b, t), read_rp_csv(b, t, false));
    }
    m
});

pub static ALG_RP: LazyLock<HashMap<(i32, i32), ALGEBRAIC_COMPARE_DATA>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for &(b, t) in algebraic_rp_truncations() {
        // Top truncated
        let (pages, _) = compute_pages(&STABLE_DATA, b, t, 0, MAX_STEM, true);
        let mut n = HashMap::new();
        for (elt, g) in STABLE_DATA.model.enumerate() {
            if let Some((af, torsion)) = pages.try_element_final(elt) {
                if torsion.alive() {
                    *n.entry((g.stem, af)).or_insert(0) += 1;
                }
            }
        }
        m.insert((b, t), n);
    }
    m
});

pub static ALG_SPHERES: LazyLock<HashMap<i32, ALGEBRAIC_COMPARE_DATA>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for sphere in 1..=MAX_STEM {
        // Top truncated
        let (pages, _) = compute_pages(&DATA, 0, sphere - 1, 0, MAX_STEM, true);
        let mut n = HashMap::new();
        for (elt, g) in DATA.model.enumerate() {
            if let Some((af, torsion)) = pages.try_element_final(elt) {
                if torsion.alive() {
                    *n.entry((g.stem, af)).or_insert(0) += 1;
                }
            }
        }
        m.insert(sphere, n);
    }
    m
});

pub fn synthetic_rp(bot_trunc: i32, top_trunc: i32) -> &'static HashMap<(i32, i32), Vec<Torsion>> {
    RP.get(&(bot_trunc, top_trunc)).expect(&format!(
        "There is no Synthetic data available for RP{bot_trunc}_{top_trunc}"
    ))
}

pub fn algebraic_rp(bot_trunc: i32, top_trunc: i32) -> &'static HashMap<(i32, i32), usize> {
    ALG_RP.get(&(bot_trunc, top_trunc)).expect(&format!(
        "There is no Algebraic data available for RP{bot_trunc}_{top_trunc}"
    ))
}

pub fn algebraic_spheres(sphere: i32) -> &'static HashMap<(i32, i32), usize> {
    ALG_SPHERES.get(&sphere).expect(&format!(
        "There is no Algebraic data available for S^{sphere}"
    ))
}

// This bot / top trunc is for compatibility with C2, which is shifted 1 down wrt. RP1_2
// So for S0, we just dont do anything with bot trunc and toptrunc
fn ahss_data_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("AHSS_DATA")
        .join(file_name)
}

fn read_csv(
    bot_trunc: i32,
    top_trunc: i32,
    file_name: &Path,
    add_one_af: bool,
    include_zero: bool,
) -> HashMap<(i32, i32), Vec<Torsion>> {
    let mut m = HashMap::new();

    if let Ok(f) = File::open(file_name) {
        for l in BufReader::new(f).lines().skip(1) {
            let s = l.unwrap();
            let spl: Vec<_> = s.split(',').collect();

            let mut stem: i32 = spl[0].parse().unwrap();
            let mut af: i32 = spl[1].parse().unwrap();
            let dr: i32 = spl[spl.len() - 1].parse().unwrap();

            if add_one_af {
                af += 1;
            }

            // These originally came not from a resolution of RPi_j
            // But from some CW spectrum, which has bottom cell in dimension 0.
            // Thus they need to be shifted
            if bot_trunc == 1 && top_trunc == 2 {
                stem += 1;
            }
            if bot_trunc == 2 && top_trunc == 4 {
                stem += 2;
            }
            if bot_trunc == 3 && top_trunc == 5 {
                stem += 3;
            }

            if stem <= MAX_VERIFY_STEM {
                if dr == 9000 || s.contains("NULL") {
                    m.entry((stem, af))
                        .or_insert(vec![])
                        .push(Torsion::default());
                } else if dr < 9000 {
                    m.entry((stem, af))
                        .or_insert(vec![])
                        .push(Torsion::new(dr - 1));
                } else if include_zero {
                    m.entry((stem, af)).or_insert(vec![]).push(Torsion::zero());
                }
            }
        }
    } else {
        panic!("Failed to open AHSS data file: {}", file_name.display())
    }
    if bot_trunc % 2 == 0 {
        for i in 0..200 {
            if let Some(p) = m.get_mut(&(bot_trunc, i)) {
                p.pop();
                if p.is_empty() {
                    m.remove(&(bot_trunc, i));
                }
            }
        }
    }
    if top_trunc % 2 == 1 {
        for i in 0..200 {
            if let Some(p) = m.get_mut(&(top_trunc, i)) {
                p.pop();
                if p.is_empty() {
                    m.remove(&(top_trunc, i));
                }
            }
        }
    }
    for j in &mut m {
        j.1.sort();
    }
    m
}

pub fn read_rp_csv(
    bot_trunc: i32,
    top_trunc: i32,
    include_zero: bool,
) -> HashMap<(i32, i32), Vec<Torsion>> {
    let file_name = ahss_data_path(&format!("RP{bot_trunc}_{top_trunc}_AdamsE2_ss.csv"));
    read_csv(bot_trunc, top_trunc, &file_name, true, include_zero)
}

// TODO:
// pub fn synthetic_stable_e1(data: &mut SyntheticSS) {
//         // Add all Synthetic stable sphere data
//     let a = vec![
//         ("6 5 3", Some(1)), ("15", Some(0)), // Stem 14/15
//         ("5 1 2 3 3", Some(2)), ("14 1", Some(0)), // Stem 14/15
//         ("3 4 4 1 1 1", Some(2)), ("13 1 1", Some(0)), // Stem 14/15
//         ("1 2 4 3 3 3", Some(1)), ("8 3 3 3", Some(0)), // Stem 16/17
//         ("2 2 4 3 3 3", Some(1)), ("5 7 3 3", Some(0)), // Stem 17/18
//         ("1 1 2 4 3 3 3", Some(1)), ("4 5 3 3 3", Some(0)), // Stem 17/18
//         ("5 1 2 3 4 4 1 1 1", Some(1)), ("13 1 2 4 1 1 1", Some(0)), // Stem 22/23
//         ("3 4 4 1 1 2 4 1 1 1", Some(1)), ("12 1 1 2 4 1 1 1", Some(0)), // Stem 22/23
//         ("1 2 4 1 1 2 4 3 3 3", Some(1)), ("8 1 1 2 4 3 3 3", Some(0)), // Stem 24/25
//         ("3 6 2 3 4 4 1 1 1", Some(1)), ("6 2 4 5 3 3 3", Some(0)), // Stem 25/26
//         ("2 2 4 1 1 2 4 3 3 3", Some(1)), ("4 2 2 4 5 3 3 3", Some(0)), // Stem 25/26
//         ("1 1 2 4 1 1 2 4 3 3 3", Some(1)), ("2 2 2 2 4 5 3 3 3", Some(0)), // Stem 25/26
//         ("4 2 2 2 4 5 3 3 3", Some(1)), ("6 2 3 5 7 3 3", Some(0)), // Stem 28/29
//         ("2 2 2 2 2 4 5 3 3 3", Some(1)), ("3 6 2 4 5 3 3 3", Some(0)), // Stem 28/29
//         ("2 2 2 2 3 5 7 3 3", Some(2)), ("12 4 5 3 3 3", Some(0)), // Stem 29/30

//         ("14 13 3", Some(1)), ("31", Some(0)), // Stem 30/31
//         ("13 11 3 3", Some(1)), ("30 1", Some(0)), // Stem 30/31
//         ("12 9 3 3 3", Some(1)), ("29 1 1", Some(0)), // Stem 30/31 // TODO: All todos this stem wrt. source, prob check wrt. EHP ?

//         ("10 2 4 5 3 3 3", Some(2)), ("28 1 1 1", Some(0)), // Stem 30/31
//         ("7 13 1 2 4 1 1 1", Some(2)), ("24 4 1 1 1", Some(0)), // Stem 30/31 // TODO
//         ("5 8 1 1 2 4 3 3 3", Some(2)), ("22 2 4 1 1 1", Some(0)), // Stem 30/31
//         ("4 2 2 2 2 4 5 3 3 3", Some(2)), ("21 1 2 4 1 1 1", Some(0)), // Stem 30/31
//         ("2 2 2 2 2 2 4 5 3 3 3", Some(2)), ("20 1 1 2 4 1 1 1", Some(0)), // Stem 30/31 // TODO

//         ("6 2 3 4 4 1 1 2 4 1 1 1", Some(3)), ("5 6 2 4 5 3 3 3", Some(0)), // Stem 30/31 // TODO
//         ("5 1 2 3 4 4 1 1 2 4 1 1 1", Some(3)), ("16 4 1 1 2 4 1 1 1", Some(0)), // Stem 30/31 // TODO
//         ("3 4 4 1 1 2 4 1 1 2 4 1 1 1", Some(3)), ("14 2 4 1 1 2 4 1 1 1", Some(0)), // Stem 30/31 // TODO

//         ("3 6 2 2 4 5 3 3 3", Some(1)), ("6 3 3 6 6 5 3", Some(0)), // Stem 31/32 // TODO: Target
//         ("2 2 2 2 2 3 5 7 3 3", Some(1)), ("3 6 2 3 5 7 3 3", Some(0)), // Stem 31/32 // TODO: Target
//         ("2 4 1 1 2 4 1 1 2 4 3 3 3", Some(3)), ("2 2 2 3 3 6 6 5 3", Some(0)), // Stem 31/32 // Proof: Needed for compatibility with RP1_2

//         ("1 2 4 1 1 2 4 1 1 2 4 3 3 3", Some(1)), ("8 1 1 2 4 1 1 2 4 3 3 3", Some(0)), // Stem 32/33

//         ("11 3 5 7 7", Some(2)), ("27 7", Some(0)), // Stem 33/34
//         ("3 6 2 3 4 4 1 1 2 4 1 1 1", Some(1)), ("6 2 2 2 2 2 4 5 3 3 3", Some(0)), // Stem 33/34
//         ("2 2 4 1 1 2 4 1 1 2 4 3 3 3", Some(1)), ("4 2 2 2 2 2 2 4 5 3 3 3", Some(0)), // Stem 33/34
//         ("1 1 2 4 1 1 2 4 1 1 2 4 3 3 3", Some(1)), ("2 2 2 2 2 2 2 2 4 5 3 3 3", Some(0)), // Stem 33/34

//         ("3 5 6 2 4 5 3 3 3", Some(1)), ("6 5 2 3 5 7 7", Some(0)), // Stem 34/35
//         ("2 2 2 2 3 3 6 6 5 3", Some(1)), ("3 6 3 3 6 6 5 3", Some(0)), // Stem 34/35

//         ("4 2 2 2 2 2 2 2 4 5 3 3 3", Some(1)), ("6 2 2 2 2 2 3 5 7 3 3", Some(0)), // Stem 36/37
//         ("2 2 2 2 2 2 2 2 2 4 5 3 3 3", Some(1)), ("3 6 2 2 2 2 2 4 5 3 3 3", Some(0)), // Stem 36/37
//         ("6 2 2 2 2 2 2 4 5 3 3 3", Some(3)), ("4 7 3 3 6 6 5 3", Some(0)), // Stem 36/37 // Proof: the source 4 7 3 3 6 6 5 3 is compatible with the Algebraic AHSS.

//         ("13 2 3 5 7 7", Some(3)), ("23 15", Some(0)), // Stem 37/38
//         ("7 12 4 5 3 3 3", Some(3)), ("22 13 3", Some(0)), // Stem 37/38
//         ("3 5 7 3 5 7 7", Some(2)), ("21 11 3 3", Some(0)), // Stem 37/38 // Proof: Target else we get a weird extra t^2 diff which need not be there. The other source would make the 39th stem not solvable.
//         ("5 6 3 3 6 6 5 3", Some(1)), ("8 12 9 3 3 3", Some(0)), // Stem 37/38 // Proof: The other option is not compatible with AEHP
//         ("3 5 6 2 3 5 7 3 3", Some(1)), ("6 9 3 6 6 5 3", Some(0)), // Stem 37/38
//         ("2 2 4 3 3 3 6 6 5 3", Some(1)), ("3 6 5 2 3 5 7 7", Some(0)), // Stem 37/38
//         ("2 2 2 2 2 2 2 2 3 5 7 3 3", Some(3)), ("2 3 5 5 3 6 6 5 3", Some(0)), // Stem 37/38

//         ("6 2 3 4 4 1 1 2 4 1 1 2 4 1 1 1", Some(3)), ("5 6 2 2 2 2 2 4 5 3 3 3", Some(0)), // Stem 38/39
//         ("5 1 2 3 4 4 1 1 2 4 1 1 2 4 1 1 1", Some(1)), ("13 1 2 4 1 1 2 4 1 1 2 4 1 1 1", Some(0)), // Stem 38/39
//         ("3 4 4 1 1 2 4 1 1 2 4 1 1 2 4 1 1 1", Some(1)), ("12 1 1 2 4 1 1 2 4 1 1 2 4 1 1 1", Some(0)), // Stem 38/39

//         ("6 2 3 4 4 1 1 2 4 1 1 2 4 1 1 1", Some(3)), ("5 6 2 2 2 2 2 4 5 3 3 3", Some(0)), // Stem 39/40
//     ];

//     for i in a {
//         for s in 1..MAX_STEM {
//             let gen_name = format!("{}[{}]", i.0, s);
//             if let Some(g) = data.find_mut(&gen_name) {
//                 g.torsion = i.1;
//             }
//         }
//     }
// }
