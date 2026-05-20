fn main() {

    let ehp = logic::get_log();

    let mut things = vec![vec![]; (MAX_STEM + 1) as usize];

    things[0].push("S^n >".to_string());

    for s in 1..MAX_STEM {
        things[0].push(format!("{s}"));
    }

    for stem in 0..MAX_STEM {
        let line_init = format!("π{stem}+n(S^n)");
        things[(stem + 1) as usize].push(line_init);
    }

    for sphere in 1..MAX_STEM {
        let top_trunc = sphere - 1;
        let (pages, _) = compute_pages(ehp, 0, top_trunc, 0, MAX_STEM, true);

        for stem in 0..MAX_STEM {
            let mut count = 0;
            for id in ehp.model.gens_id_in_stem(stem) {
                if let Some(el) = pages.try_element_final(*id)
                    && el.1.free()
                {
                    count += 1;
                }
            }

            things[(stem + 1) as usize].push(format!("{}", count));
        }
    }

    let joined: Vec<_> = things.into_iter().map(|x| x.join(",")).collect();
    for l in joined {
        println!("{l}");
    }
}