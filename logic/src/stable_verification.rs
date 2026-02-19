use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}};

use itertools::Itertools;

use crate::{MAX_VERIFY_SPHERE, MAX_VERIFY_STEM, processor::get_filtered_data, types::{Category, SyntheticSS}};

fn read_rp_even_csv(bottom_trunc: i32, top_trunc: i32) -> HashMap<(i32, i32), Vec<Option<i32>>> {
    let file_name = format!("../RP_ASS/RP{bottom_trunc}_{top_trunc}_AdamsE2_ss.csv",);

    let mut m = HashMap::new();

    if let Ok(f) = File::open(file_name) {
        for l in BufReader::new(f).lines().skip(1) {
            let s = l.unwrap();
            let spl: Vec<_> = s.split(',').collect();

            let mut stem: i32 = spl[0].parse().unwrap();
            let mut af: i32 = spl[1].parse().unwrap();
            let dr: i32 = spl[spl.len()-1].parse().unwrap();

            af += 1;

            if bottom_trunc == 1 && top_trunc == 2 {
                stem += 1;
            }

            if stem <= MAX_VERIFY_STEM {
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

    m
} 

fn read_rp_uneven_csv(bottom_trunc: i32, top_trunc: i32) -> HashMap<(i32, i32), Vec<Option<i32>>> {
    let file_name = format!("../RP_ASS/RP{bottom_trunc}_{top_trunc}_AdamsE2_ss.csv",);

    let mut m = HashMap::new();

    if let Ok(f) = File::open(file_name) {
        for l in BufReader::new(f).lines().skip(1) {
            let s = l.unwrap();
            let spl: Vec<_> = s.split(',').collect();

            let mut s: i32 = spl[1].parse().unwrap();
            let t: i32 = spl[2].parse().unwrap();
            let dr: i32 = spl[spl.len()-1].parse().unwrap();
            let w = spl[spl.len()-2];

            let stem = t - s;
            s += 1;

            if stem <= MAX_VERIFY_STEM {
                if dr == 9000 || w == "" {
                    m.entry((stem, s)).or_insert(vec![]).push(None);
                } 
                else if dr < 9000 {
                    m.entry((stem, s)).or_insert(vec![]).push(Some(dr - 1));
                }
            }
        }
    } else {
        panic!()
    }

    m
} 

fn read_rp_csv(bottom_trunc: i32, top_trunc: i32)  -> HashMap<(i32, i32), Vec<Option<i32>>> {
    if top_trunc & 1 == 1 {
        read_rp_uneven_csv(bottom_trunc, top_trunc)
    } else {
        read_rp_even_csv(bottom_trunc, top_trunc)
    }
}


pub fn verify_rp(data: &SyntheticSS) -> bool {
    const UNEVEN_MAX_AF_Z_COPY: i32 = 30;
    
    let mut is_valid = true;

    for upper_trunc in 2..=MAX_VERIFY_SPHERE {
        if upper_trunc & 1 == 1 && upper_trunc > 11 {
            continue;
        }
        
        let mut expected = read_rp_csv(1, upper_trunc);

        let gens = get_filtered_data(data, Category::Synthetic, 1, upper_trunc + 1, 1000, None);

        let mut compare = HashMap::new();
        
        for n in gens {
            let g = data.find(&n.0).unwrap();
            let (stem, af, dr) = (g.x, n.1.1, n.1.0);
            if 0 < g.x && g.x <= MAX_VERIFY_STEM && n.1.0 != Some(0) && g.adams_filtration <= UNEVEN_MAX_AF_Z_COPY {
                compare.entry((stem, af)).or_insert(vec![]).push(dr);
            }
        }

        if upper_trunc & 1 == 1 {
            for j in 3..=UNEVEN_MAX_AF_Z_COPY {
                compare.entry((upper_trunc, j)).or_insert(vec![]).push(None);
            }
        }

        for j in &mut expected {
            j.1.sort();
        }
        for j in &mut compare {
            j.1.sort();
        }

        for (stem_af, gens) in expected.iter().sorted() {
            if stem_af.1 > UNEVEN_MAX_AF_Z_COPY {
                continue;
            }

            if !compare.contains_key(stem_af) {
                eprintln!("In stem {}, af {}, we have unequal generators for RP_1^{upper_trunc}. Expect: {:?} | Have: {:?}", stem_af.0, stem_af.1, gens, 0);
                is_valid = false;
            } else {
                if compare.get(stem_af).unwrap() != gens {
                    eprintln!("In stem {}, af {}, we have unequal generators for RP_1^{upper_trunc}. Expect: {:?} | Have: {:?}", stem_af.0, stem_af.1, gens, compare.get(stem_af).unwrap());
                    is_valid = false;
                }
            }
        }

        for (stem_af, g) in &compare {
            if stem_af.1 > UNEVEN_MAX_AF_Z_COPY {
                continue;
            }
            if !expected.contains_key(stem_af) {
                eprintln!("In stem {}, af {}, we have unequal generators for RP_1^{upper_trunc}. Expect: {:?} | Have: {:?}", stem_af.0, stem_af.1, 0, g);
                is_valid = false;
            } 
        }
    }
    is_valid
}

pub fn verify_ehp_to_ahss(ehp: &SyntheticSS, ahss: &SyntheticSS) {
    // Check if the map respect Z[tau] possibilities. on E1
    for ehp_g in &ehp.generators {
        if ehp_g.x <= MAX_VERIFY_STEM {
            if let Some(ahss_g) = ahss.find(&ehp_g.name) {
                if ahss_g.adams_filtration != ehp_g.adams_filtration {
                    eprintln!("Adams filtrations do not agree for {}. In stem {} | y {}", ehp_g.name, ehp_g.x, ehp_g.y)
                }
                if ahss_g.adams_filtration != ehp_g.adams_filtration {
                    eprintln!("Torsion does not agree for {}. In stem {} | y {}", ehp_g.name, ehp_g.x, ehp_g.y)
                }
            }
        }
    }

    // For all pages check the diffs
    for page in 1..=MAX_VERIFY_STEM {
        
        let ehp_gens = get_filtered_data(ehp, Category::Synthetic, 1, 1000, page, None);
        let ahss_gens = get_filtered_data(ahss, Category::Synthetic, 1, 1000, page, None);
        
        // Check if dr commutes
        for d in &ehp.differentials {
            if d.d == page {
                let e_from = ehp_gens.get(&d.from).unwrap();
                let e_to = ehp_gens.get(&d.to).unwrap();


                if ehp.find(&d.from).unwrap().x > MAX_VERIFY_STEM {
                    continue;
                }
                
                // Check if the from maps are legal
                if let Some(a_from) = ahss_gens.get(&d.from) {
                    if let Some(e_t) = e_from.0 {
                        if e_t != 0 {
                            if let Some(a_t) = a_from.0 {
                                if a_t > e_t {
                                    eprintln!("For the ehp diff {:?}, we have that the FROM it is not compatible. EHP from: {:?} | AHSS from: {:?}", d, e_from, a_from);
                                } 
                            } else {
                                eprintln!("For the ehp diff {:?}, we have that the FROM it is not compatible. EHP from: {:?} | AHSS from: {:?}", d, e_from, a_from);
                            }
                        }
                    }
                }

                // Check if the to maps are legal
                if let Some(a_to) = ahss_gens.get(&d.from) {
                    if let Some(e_t) = e_to.0 {
                        if e_t != 0 {
                            if let Some(a_t) = a_to.0 {
                                if a_t > e_t {
                                    eprintln!("For the ehp diff {:?}, we have that the FROM it is not compatible. EHP from: {:?} | AHSS from: {:?}", d, e_to, a_to);
                                } 
                            } else {
                                eprintln!("For the ehp diff {:?}, we have that the FROM it is not compatible. EHP from: {:?} | AHSS from: {:?}", d, e_to, a_to);
                            }
                        }
                    }
                }
                
                // Check if there is a diff in the target
                if let Some(a_to) = ahss_gens.get(&d.to) {
                    if let Some(a_from) = ahss_gens.get(&d.from) {
                        if &d.from == "7 1 1[4]" {
                            println!("{:?}", a_from);
                        }
                        let mut found = false;
                        for a_d in &ahss.differentials {
                            if a_d.d != page {
                                continue;
                            }
                            if a_d.from == d.from && a_d.to == d.to {
                                // Found a diff
                                found = true;
                                if a_d.coeff != d.coeff {
                                    eprintln!("uhmmmmm"); // Could happen if we have multiple diffs from the same page ?
                                }
                            }
                        }
                        if !found && a_from.0 != Some(0) && a_to.0 != Some(0) {
                            if let Some(a_to_t) = a_to.0 {
                                if a_to_t > d.coeff {
                                    eprintln!("For the ehp diff {:?}, there does not exists a ahss diff, although there are valid targets", d);
                                }
                            } else {
                                eprintln!("For the ehp diff {:?}, there does not exists a ahss diff, although there are valid targets", d);
                            }
                        }
                    } else {
                        eprintln!("For the ehp diff {:?}, that there exists a to in ahss, but no from in ahss.", d);
                    }
                }
            }
        }   


        // Check if some lifts should exist
        for d in &ahss.differentials {
            
        }


    }
}