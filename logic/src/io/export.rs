use std::{fs::File, io::{self, Read, Write}};

use serde::{Deserialize, Serialize};

use crate::{MAX_STEM, domain::process::compute_pages, domain::model::{SSPages, SyntheticSS}, solve::action::Action, types::Kind};

pub fn write_vec_to_file<T: std::fmt::Debug>(vec: &[T], path: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    for item in vec {
        writeln!(file, "{item:?}")?;
    }

    Ok(())
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Differential {
    pub from: String,
    pub to: String,
    pub coeff: i32,
    pub d: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,

    pub kind: Kind,
}

impl PartialEq for Differential {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.coeff == other.coeff && self.d == other.d && self.proof == other.proof
    }
}

impl Eq for Differential { }

impl PartialOrd for Differential {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.d == other.d {
            return other.coeff.partial_cmp(&self.coeff);
        } else {
            return self.d.partial_cmp(&other.d);
        }
    }
}

impl Ord for Differential {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.d == other.d {
            return other.coeff.cmp(&self.coeff);
        } else {
            return self.d.cmp(&other.d);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauMult {
    pub from: String,
    pub to: String,
    pub kind: Kind,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multiplication {
    pub from: String,
    pub to: String,
    pub internal: bool,
    pub kind: String,
}


pub fn write_typescript_file(
    output_path: &str,
    data_name: &str,
    data: &SyntheticSS,
    pages: &SSPages,
) -> Result<(), std::io::Error> {
    let mut file = File::create(output_path)?;

    // Serialize generators to JSON strings
    let gens: Vec<String> = data.model.gens()
        .iter()
        .map(|g| serde_json::to_string(g).unwrap())
        .collect();

    let mut differentials = vec![];
    for ds in &data.diffs_page {
        for d in ds {
            let d_y = data.model.y(d.from) - data.model.y(d.to);

            let from_name = data.model.name(d.from).to_string();
            let to_name = data.model.name(d.to).to_string();

            
            let from_af = if pages.element_in_pages(d.from) {
                pages.element_at_page(d_y, d.from).0
            } else {
                data.model.original_af(d.from)
            };
            let to_af = data.model.original_af(d.to);
            
            let coeff = to_af - from_af - 1;

            let proof = data.proven_from_to.get(&(d.from,d.to)).unwrap().clone();

            let diff = Differential {
                from: from_name,
                to: to_name,
                coeff,
                d: d_y,
                proof,
                kind: Kind::Real,
            };
            differentials.push(diff);
        }
    }

    let mut int_tau_mults = vec![];
    for p in &data.internal_tau_page {
        for i_t in p {
            let from = data.model.name(i_t.from);
            let to = data.model.name(i_t.to);
            let proof = data.proven_from_to.get(&(i_t.from, i_t.to)).unwrap_or(&None).clone();
            int_tau_mults.push(
                TauMult {
                    from: from.to_string(),
                    to: to.to_string(),
                    kind: Kind::Real,
                    proof
                }
            );
        } 
    }

    let mut ext_tau_mults = vec![];
    for e_t in &data.external_tau_page {
        let from = data.model.name(e_t.from);
        let to = data.model.name(e_t.to);
        let proof = data.proven_from_to.get(&(e_t.from, e_t.to)).unwrap_or(&None).clone();
        ext_tau_mults.push(
            TauMult {
                from: from.to_string(),
                to: to.to_string(),
                kind: Kind::Real,
                proof
            }
        );
    }
    
    for (from,to) in &data.temp_fakes {
        let d_y = data.model.y(*from) - data.model.y(*to);
        let d_stem = data.model.stem(*from) - data.model.stem(*to);
        if d_y == 0 {
            int_tau_mults.push(TauMult { from:  data.model.name(*from).to_string(), to: data.model.name(*to).to_string(), kind: Kind::Unknown, proof: Some(format!("Unknown")) });
        } else if d_stem == 0 {
            ext_tau_mults.push(TauMult { from:  data.model.name(*from).to_string(), to: data.model.name(*to).to_string(), kind: Kind::Unknown, proof: Some(format!("Unknown")) });
        } else {
            let from_af = pages.element_at_page(d_y, *from).0;
            let to_af = data.model.original_af(*to);
            
            let coeff = to_af - from_af - 1;            
            differentials.push(Differential { from:  data.model.name(*from).to_string(), to: data.model.name(*to).to_string(), kind: Kind::Unknown, proof: Some(format!("Unknown")), coeff, d: d_y });
        }
    }

    // Serialize differentials to JSON strings
    let diffs: Vec<String> = differentials
        .iter()
        .map(|d| serde_json::to_string(d).unwrap())
        .collect();

    let int_tau_mults: Vec<String> = int_tau_mults
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();

    let mut ext_tau_mults: Vec<String> = ext_tau_mults
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
    
    let mults: Vec<String> = vec![];

    let mut tau_mults: Vec<String> = int_tau_mults;
    tau_mults.append(&mut ext_tau_mults);

    let pre = format!(
        "// This file has been generated by curtis.rs\n\
         // Based on curtis tables of the AEHP sequence\n\
         import {{ SyntheticEHP }} from \"./types\";\n\n\
         export const MAX_STEM{} = {};\n\n\
         export const data{}: SyntheticEHP = {{\n\
         \x20   \"generators\": [\n",
        data_name.to_uppercase(), MAX_STEM, data_name
    );

    let ds = "\n    ],\n    \"differentials\": [\n";
    let ms = "\n    ],\n    \"multiplications\": [\n";
    let ts = "\n    ],\n    \"tau_mults\": [\n";
    let post = "    ]\n}";

    file.write_all(pre.as_bytes())?;
    file.write_all(gens.join(",\n").as_bytes())?;
    file.write_all(ds.as_bytes())?;
    file.write_all(diffs.join(",\n").as_bytes())?;
    file.write_all(ms.as_bytes())?;
    file.write_all(mults.join(",\n").as_bytes())?;
    file.write_all(ts.as_bytes())?;
    file.write_all(tau_mults.join(",\n").as_bytes())?;
    file.write_all(post.as_bytes())?;

    Ok(())
}


pub fn get_log(ahss: bool) -> Result<Vec<Action>, ()> {
    let mut f = if ahss {
        File::open("../log_stable.json").map_err(|_| ())?
    } else {
        File::open("../log.json").map_err(|_| ())?
    };
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    serde_json::de::from_str(&s).map_err(|_| ())
}

pub fn write_all(data: &SyntheticSS, log: &Vec<Action>, ahss: bool) {
    if ahss {
        let (pages, _) = compute_pages(&data, 0, 256, 0, MAX_STEM);
        write_typescript_file("../site/src/data_stable.ts", "_stable", &data, &pages).unwrap();
        write_log(&log,ahss).unwrap();
    } else {
        let (pages, _) = compute_pages(&data, 0, 256, 0, MAX_STEM);
        write_typescript_file("../site/src/data.ts", "", &data, &pages).unwrap();
        write_log(&log,ahss).unwrap();
    }
}

pub fn write_log(log: &Vec<Action>, ahss: bool) -> io::Result<()> {
    let name = if ahss {
        "log_stable"
    } else {
        "log"
    };
    
    write_vec_to_file(&log, &format!("../{}.txt", name), )?;
    let mut file = File::create(&format!("../{}.json", name))?;
    writeln!(file, "{}", serde_json::to_string(log)?)?;
    file.flush().unwrap();
    Ok(())
}