use std::collections::HashMap;


/// Convert algebraic tags to human-readable names
/// This is currently unused but kept for potential future use
#[allow(dead_code)]
fn tag_to_name(n: &str, x: i32, y: i32, af: i32) -> Option<String> {
    let original = n.to_string();
    let mut output = String::new();
    let mut remaining = n.to_string();

    let map: HashMap<&str, &str> = [
        ("1 2 3 4 4 1 1 1", "h_1Pc_0"),
        ("1 1 2 4 3 3 3", "h_0^3e_0"),
        ("2 3 4 4 1 1 1", "Pc_0"),
        ("1 2 4 3 3 3", "h_1^2d_0"),
        ("2 2 4 3 3 3", "h_0^2e_0"),
        ("2 4 3 3 3", "h_1d_0"),
        ("4 4 1 1 1", "Ph_2"),
        ("1 1 2 4 1 1 1", "h_0^2Ph_2"),
        ("2 2 4 1 1 1", "h_0Ph_2"),
        ("1 2 4 1 1 1", "h_1Ph_1"),
        ("2 4 1 1 1", "Ph_1"),
        ("5 1 2 3 3", "h_0d_0"),
        ("1 2 3 3", "h_1c_0"),
        ("2 4 1 1", "P"),
        ("6 2 3 3", "d_0"),
        ("4 1 1 1", "h_0^3h_3"),
        ("6 5 3", "h_0h_3^2"),
        ("2 3 3", "c_0"),
        ("3 3 3", "h_1^2h_3"),
        ("5 1 1", "h_0^2h_3"),
        ("1 1 1", "h_0^2h_2"),
        ("5 3", "h_1h_3"),
        ("6 1", "h_0h_3"),
        ("2 1", "h_0h_2"),
        ("4 5", "h_0f_0"),
        ("3 6", "h_0e_0"),
        ("3 3", "h_2^2"),
        ("1 1", "h_1^2"),
        ("7", "h_3"),
        ("3", "h_2"),
        ("1", "h_1"),
        ("0", "h_0"),
    ].iter().cloned().collect();

    loop {
        let mut flag = false;
        for (comp, ans) in &map {
            if remaining.ends_with(comp) {
                output.push_str(ans);
                remaining = remaining.trim_end_matches(comp).trim_end().to_string();
                flag = true;
                break;
            }
        }
        if !flag {
            eprintln!("{} | {} {} {}", original, x, y, af);
            return None;
        }
        if remaining.is_empty() {
            break;
        }
    }

    Some(output)
}
