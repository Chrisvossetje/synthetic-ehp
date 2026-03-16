use crate::domain::model::SyntheticSS;




fn in_metastable_range(y: i32, stem: i32) -> bool {
    stem < (y*3)
}

pub fn set_metastable_range(ehp: &mut SyntheticSS, ahss: &SyntheticSS) -> Result<(),()> {
    for g in ahss.model.gens() {
        if in_metastable_range(g.y, g.stem) {
            ehp.set_generator(&g.name, g.torsion)?;
        }
    }
    for ds in &ahss.diffs_page {
        for d in ds {
            let g_from = ahss.model.get(d.from);
            let g_to = ahss.model.get(d.to);
            if in_metastable_range(g_to.y, g_to.stem) {
                let proof = ahss.proven_from_to.get(&(d.from, d.to)).expect("If there is no reference to a proof here (note that internally it can still have no proof), then inserting differentials not done carefully enough.");
                ehp.add_diff_name(g_from.name.clone(), g_to.name.clone(), proof.clone())?;
            }
        }
    }
    for (page, ts) in ahss.internal_tau_page.iter().enumerate() {
        for t in ts {
            let g_from = ahss.model.get(t.from);
            let g_to = ahss.model.get(t.to);
            if in_metastable_range(g_to.y, g_to.stem) {
                let proof = ahss.proven_from_to.get(&(t.from, t.to)).expect("If there is no reference to a proof here (note that internally it can still have no proof), then inserting internal tau's not done carefully enough.");
                ehp.add_int_tau_name(g_from.name.clone(), g_to.name.clone(), page as i32, proof.clone())?;
            }
        }
    }

    for e in &ahss.external_tau_page {
        let g_from = ahss.model.get(e.from);
        let g_to = ahss.model.get(e.to);
        if in_metastable_range(g_to.y, g_to.stem) {
            let proof = ahss.proven_from_to.get(&(e.from, e.to)).expect("If there is no reference to a proof here (note that internally it can still have no proof), then inserting external tau's not done carefully enough.");
            ehp.add_ext_tau_name(g_from.name.clone(), g_to.name.clone(), proof.clone())?;
        }
    }


    Ok(())
}