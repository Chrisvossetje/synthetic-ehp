type Monomial = Vec<u8>;


// ==========================
// Binomial modulo 2 (Lucas lemma)
// ==========================
fn binom_bool(m: u8, n: u8) -> bool {
    // Using Lucas theorem modulo 2: C(n,m) odd <=> m & n == m
    (m & n) == m
}

// ==========================
// Adem relations: reduce-pair
// ==========================
fn reduce_pair(i: u8, k: u8) -> Vec<Monomial> {
    if 2*i >= k {
        return vec![vec![i, k]];
    }
    let n = k - 2 * i - 1;
    println!("in: {i}, {k} | n: {n}");
    let mut result = Vec::new();
    if n == 0 {
        return vec![];
    }
    for j in 0..=((n + 1) / 2 - 1) {
        if binom_bool(n - (j + 1), j) {
            result.push(vec![i + n - j, 1 + 2 * i + j]);
        }
    }
    result
}

// ==========================
// Differential generator
// ==========================
fn differential_gen(b: u8) -> Vec<Monomial> {
    let mut result = Vec::new();
    for c in 0..=((b + 1) / 2 - 1) {
        let d = b - c - 1;
        if binom_bool(d, c + 1) {
            result.push(vec![d, c]);
        }
    }
    result
}

// ==========================
// Leibniz rule on monomial
// ==========================
fn leibniz_mon(mon: &[u8]) -> Vec<Monomial> {
    let mut result = Vec::new();
    for (n, &val) in mon.iter().enumerate() {
        for r in differential_gen(val) {
            let mut new_mon = Vec::new();
            new_mon.extend_from_slice(&mon[0..n]);
            new_mon.extend(r);
            new_mon.extend_from_slice(&mon[n + 1..]);
            result.push(new_mon);
        }
    }
    result
}

// Leibniz rule on polynomial
fn leibniz_poly(poly: &[Monomial]) -> Vec<Monomial> {
    poly.iter().flat_map(|m| leibniz_mon(m)).collect()
}

// ==========================
// Admissibility check
// ==========================
fn admissible_place(n: usize, mon: &[u8]) -> bool {
    2 * mon[n - 1] >= mon[n]
}

// ==========================
// Multiplicative reduction
// ==========================
fn mult_reduce_poly(mut poly: Vec<Monomial>) -> Vec<Monomial> {
    let mut result = Vec::new();

    while let Some(mon) = poly.pop() {
        let mut reduced = true;
        for i in 1..mon.len() {
            if !admissible_place(i, &mon) {
                let new_mons = reduce_pair(mon[i - 1], mon[i]);
                for nm in new_mons {
                    let mut new_mon = mon[..i - 1].to_vec();
                    new_mon.extend(nm);
                    new_mon.extend_from_slice(&mon[i + 1..]);
                    poly.push(new_mon);
                }
                reduced = false;
                break;
            }
        }
        if reduced {
            result.push(mon);
        }
    }

    result
}

// ==========================
// Lexicographic order & equality
// ==========================
fn mon_leq(mon1: &[u8], mon2: &[u8]) -> bool {
    for (i, j) in mon1.iter().zip(mon2.iter()) {
        if i < j {
            return true;
        } else if i > j {
            return false;
        }
    }
    true
}

fn mon_equal(mon1: &[u8], mon2: &[u8]) -> bool {
    mon1 == mon2
}

// ==========================
// Additive reduction
// ==========================
fn remove_consecutive_dupes(poly: Vec<Monomial>) -> Vec<Monomial> {
    let mut result = Vec::new();
    let mut iter = poly.into_iter();
    if let Some(mut prev) = iter.next() {
        result.push(prev.clone());
        for mon in iter {
            if !mon_equal(&prev, &mon) {
                result.push(mon.clone());
                prev = mon;
            }
        }
    }
    result
}

fn additively_reduce_poly(mut poly: Vec<Monomial>) -> Vec<Monomial> {
    poly.sort_by(|a, b| b.cmp(a)); // descending lex
    remove_consecutive_dupes(poly)
}

// ==========================
// Fully reduce polynomial
// ==========================
fn reduce_poly(poly: Vec<Monomial>) -> Vec<Monomial> {
    additively_reduce_poly(mult_reduce_poly(poly))
}

// ==========================
// Differential on polynomial
// ==========================
fn differential_poly(poly: Vec<Monomial>) -> Vec<Monomial> {
    reduce_poly(leibniz_poly(&poly))
}

// ==========================
// Polynomial concatenation
// ==========================
fn concatenate_polys(poly1: &[Monomial], poly2: &[Monomial]) -> Vec<Monomial> {
    let mut result = Vec::new();
    for m1 in poly1 {
        for m2 in poly2 {
            let mut new_mon = m1.clone();
            new_mon.extend(m2);
            result.push(new_mon);
        }
    }
    result
}

fn concatenate_mon_to_poly(mon: &[u8], poly: &[Monomial]) -> Vec<Monomial> {
    poly.iter()
        .map(|m2| {
            let mut new_mon = mon.to_vec();
            new_mon.extend(m2);
            new_mon
        })
        .collect()
}



fn main() {
    println!("{:?}", reduce_pair(0, 3));
    println!("{:?}", differential_gen(1));
    println!("{:?}", differential_poly(vec![vec![0,3]]));
}
