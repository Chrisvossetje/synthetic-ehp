use std::{iter::FilterMap, ops::RangeInclusive};

use crate::{
    MAX_STEM, MAX_VERIFY_STEM,
    data::compare::{S0, algebraic_spheres},
    domain::{
        model::SyntheticSS,
        process::{compute_pages, ehp_recursion, try_compute_pages},
        ss::SSPages,
    },
    solve::{
        ehp_ahss::{SyntheticSSMap, compare_ehp_ahss},
        issues::{
            Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic,
        },
    },
};

pub fn ehp_stable_verify(
    ehp: &SyntheticSS,
    ahss: &SyntheticSS,
    pages: &SSPages,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    // let a = ahss.model.gens_id_in_stem_y(stem+1, 1);
    // let expected: HashMap<_, _> = a.iter().map(|i| {
    //     let g = ahss.model.get(*i);
    //     (g.af, g.torsion)
    // }).collect();
    // let expected_names: HashSet<_> = a.iter().map(|i| {
    //     let g = ahss.model.get(*i);
    //     &g.name
    // }).collect();

    let observed = pages.convergence_at_stem(ehp, stem);

    compare_synthetic(&observed, &S0, 0, 256, stem)?;

    Ok(())
}

fn verify_algebraic_convergence(
    data: &SyntheticSS,
    pages: &SSPages,
    sphere: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let observed = pages.algebraic_convergence_at_stem(data, stem);

    compare_algebraic(&observed, algebraic_spheres(sphere), 0, sphere, stem)
}

fn ehp_iterate(
    stem_minus_sphere: i32,
    slanted: bool,
) -> FilterMap<RangeInclusive<i32>, impl FnMut(i32) -> Option<(i32, i32)>> {
    (1..=(MAX_STEM)).filter_map(move |i| {
        let stem = if slanted {
            stem_minus_sphere - (i / 2)
        } else {
            stem_minus_sphere
        };
        let sphere = 1 + i;
        if stem < 2 {
            None
        } else if stem > MAX_STEM {
            None
        } else if sphere - 2 > stem {
            // Stable range already seen
            None
        } else {
            Some((stem, sphere))
        }
    })
}

pub fn apply_ehp_recursively(
    ehp: &mut SyntheticSS,
    stem_minus_sphere: i32,
    slanted: bool,
) -> Result<(), Vec<Issue>> {
    for (stem, sphere) in ehp_iterate(stem_minus_sphere, slanted).rev() {
        if sphere % 2 != 1 {
            continue;
        }
        if sphere - 2 >= stem {
            // Stable
            continue;
        }
        if (sphere + 2) / 2 + stem >= MAX_STEM {
            continue;
        }
        ehp_recursion(ehp, sphere, stem)?;
    }

    Ok(())
}

pub fn find_ehp_issues(
    ehp: &mut SyntheticSS,
    ahss: &SyntheticSS,
    map: &SyntheticSSMap,
    stem_minus_sphere: i32,
    slanted: bool,
) -> Result<(), Vec<Issue>> {
    // If there is an issue i prefer to first check the algebraic convergence though
    match apply_ehp_recursively(ehp, stem_minus_sphere, slanted) {
        Ok(_) => {}
        Err(issues) => {
            let (sphere, stem) = match &issues[0] {
                Issue::InvalidName {
                    original_name,
                    unexpected_name,
                    sphere,
                    stem,
                    af,
                } => (*sphere, *stem),
                _ => {
                    return Err(issues);
                }
            };

            let pages = try_compute_pages(ehp, 0, sphere - 1, stem - 1, stem, true)?;

            verify_algebraic_convergence(ehp, &pages, sphere, stem)?;

            return Err(issues);
        }
    }

    for (stem, sphere) in ehp_iterate(stem_minus_sphere, slanted) {
        if stem > MAX_VERIFY_STEM {
            continue;
        }

        let pages = if sphere - 2 == stem {
            // Stable
            let pages = try_compute_pages(ehp, 0, sphere - 1, stem, stem, true)?;
            ehp_stable_verify(ehp, ahss, &pages, stem)?;
            pages
        } else {
            // Unstable
            let pages = try_compute_pages(ehp, 0, sphere - 1, stem - 1, stem, true)?;
            verify_algebraic_convergence(ehp, &pages, sphere, stem)?;
            pages
        };

        compare_algebraic_spectral_sequence(ehp, &pages, stem, 0, sphere - 1, false)?;
        compare_ehp_ahss(ehp, ahss, map, stem, sphere - 1)?;
    }

    Ok(())
}

/// Verify stable stems match expected values
pub fn verify_geometric(data: &SyntheticSS) {
    const CLASSICAL_MAX_SPHERE: i32 = 35;
    const CLASSICAL_MAX_STEM: i32 = 33;

    // Row n = pi_n+*(S^*)
    // Col k = pi_*+k(S^k)
    let geometric_gens: [[i32; CLASSICAL_MAX_SPHERE as usize]; (CLASSICAL_MAX_STEM + 1) as usize] = [
        [
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        ],
        [
            1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        ],
        [
            1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        ],
        [
            1, 2, 12, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        ],
        [
            1, 12, 2, 4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ],
        [
            1, 2, 2, 4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ],
        [
            1, 2, 3, 72, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        ],
        [
            1, 3, 15, 15, 30, 60, 120, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240,
            240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240,
        ],
        [
            1, 15, 2, 2, 2, 48, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 2, 4, 8, 8, 8, 16, 32, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8,
        ],
        [
            1, 4, 24, 2880, 144, 144, 48, 1152, 48, 24, 12, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        ],
        [
            1, 24, 336, 2688, 2016, 2016, 1008, 1008, 1008, 504, 504, 504, 504, 504, 504, 504, 504,
            504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504,
            504,
        ],
        [
            1, 336, 4, 64, 8, 240, 1, 1, 1, 12, 2, 4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
        ],
        [
            1, 4, 6, 288, 12, 6, 6, 12, 6, 6, 12, 12, 6, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3,
        ],
        [
            1, 6, 30, 30240, 12, 24, 96, 23040, 64, 32, 32, 384, 32, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 30, 30, 30, 60, 360, 960, 3840, 1920, 960, 480, 480, 960, 960, 960, 960, 960, 960,
            960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960,
        ],
        [
            1, 30, 12, 72, 4, 2016, 16, 128, 16, 480, 2, 2, 2, 48, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 12, 48, 4608, 16, 16, 16, 96, 16, 8, 8, 16, 16, 16, 32, 64, 32, 16, 16, 16, 16, 16,
            16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
        ],
        [
            1, 48, 48, 46080, 96, 288, 48, 24192, 48, 96, 64, 15360, 128, 128, 128, 3072, 128, 64,
            32, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
        ],
        [
            1, 48, 264, 4224, 528, 8448, 528, 528, 528, 1584, 2112, 8448, 2112, 2112, 1056, 1056,
            1056, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528,
            528, 528,
        ],
        [
            1, 264, 4, 64, 24, 5760, 24, 72, 24, 12096, 96, 768, 192, 5760, 24, 24, 24, 288, 48,
            96, 48, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        ],
        [
            1, 4, 2, 32, 4, 2, 4, 32, 8, 8, 16, 32, 32, 16, 8, 16, 8, 8, 16, 16, 8, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 2, 2, 32, 8, 32, 64, 4096, 128, 64, 64, 512, 64, 128, 128, 2048, 128, 64, 64, 256,
            32, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 2, 4, 32, 32, 2048, 1024, 8192, 2048, 2048, 512, 512, 512, 512, 2048, 8192, 2048,
            1024, 512, 512, 512, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256,
            256,
        ],
        [
            1, 4, 8, 32, 16, 512, 128, 4096, 128, 1024, 32, 16, 16, 128, 64, 512, 64, 128, 8, 4, 4,
            16, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 8, 32, 2048, 128, 1024, 256, 32768, 256, 64, 64, 128, 32, 32, 64, 128, 64, 16, 8, 4,
            4, 4, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 32, 32, 32768, 256, 2048, 512, 540672, 512, 256, 128, 8192, 128, 32, 32, 256, 32,
            32, 32, 256, 32, 32, 32, 256, 32, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
        [
            1, 32, 32, 4096, 128, 1024, 128, 3072, 128, 256, 256, 512, 128, 32, 8, 8, 8, 8, 32,
            128, 32, 32, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        ],
        [
            1, 32, 16, 4096, 32, 512, 16, 128, 32, 128, 64, 256, 16, 64, 2, 2, 2, 16, 8, 64, 16,
            32, 2, 2, 2, 8, 4, 8, 4, 2, 2, 2, 2, 2, 2,
        ],
        [
            1, 16, 8, 4096, 64, 64, 128, 16384, 256, 512, 64, 256, 16, 4, 4, 16, 4, 4, 8, 16, 16,
            8, 4, 4, 2, 2, 4, 4, 2, 1, 1, 1, 1, 1, 1,
        ],
        [
            1, 8, 8, 1024, 64, 256, 512, 1048576, 1024, 1024, 128, 512, 64, 256, 256, 16384, 256,
            64, 64, 512, 64, 128, 128, 1024, 64, 32, 32, 128, 16, 8, 4, 2, 2, 2, 2,
        ],
        [
            1, 8, 16, 256, 32, 1024, 128, 8192, 256, 512, 32, 128, 64, 1024, 2048, 8192, 2048,
            1024, 256, 256, 256, 256, 1024, 4096, 1024, 512, 256, 256, 256, -2, -2, -2, -2, -2, -2,
        ],
        [
            1, 16, 16, 2048, 32, 2048, 64, 4096, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2,
            -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2,
        ],
        [
            1, 16, 8, 4096, 16, 128, 64, 2048, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2,
            -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2,
        ],
    ];

    for sphere in (1..=CLASSICAL_MAX_SPHERE).into_iter().rev() {
        let (pages, _) = compute_pages(data, 0, sphere - 1, 0, 256, true);

        let mut conv_gens = vec![0; (CLASSICAL_MAX_STEM + 1) as usize];

        for (id, g) in data.model.enumerate() {
            if pages.element_in_pages(id) {
                if pages.element_final(id).1.free() {
                    if g.stem <= CLASSICAL_MAX_STEM {
                        conv_gens[g.stem as usize] += 1;
                    }
                }
            }
        }

        let geometric_order: Vec<_> = (0..=CLASSICAL_MAX_STEM)
            .into_iter()
            .map(|x| geometric_gens[x as usize][(sphere - 1) as usize].trailing_zeros())
            .collect();

        for stem in 0..=CLASSICAL_MAX_STEM {
            if geometric_order[stem as usize] != conv_gens[stem as usize] {
                if geometric_gens[stem as usize][(sphere - 1) as usize] != -2 {
                    eprintln!(
                        "Geometric homotopy groups on the {sphere} Sphere do not agree on stem {stem}. Expect: {}, Got: {}",
                        geometric_order[stem as usize], conv_gens[stem as usize]
                    );
                }
            }
        }
    }
}
