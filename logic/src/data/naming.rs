use std::{collections::HashMap, iter::{Map, StepBy}, ops::RangeInclusive};

use crate::MAX_STEM;


pub fn generate_names_from_tag(tag: &str) -> Map<RangeInclusive<i32>, impl FnMut(i32) -> String> {
    (1..=MAX_STEM).map(move |x| format!("{tag}[{x}]"))
}

pub fn generate_names_from_tag_special(tag: &str, start: i32, step: usize) -> Map<StepBy<RangeInclusive<i32>>, impl FnMut(i32) -> String> {
    (start..=MAX_STEM).step_by(step).map(move |x| format!("{tag}[{x}]"))
}

pub fn add_sphere_to_tag(mut tag: String, sphere: i32) -> String {
    tag.push('[');
    tag.push_str(&((sphere - 1) / 2).to_string());
    tag.push(']');
    tag
}

/// Extract the generating name from a generator
/// Example: "5 3[6]" -> "5 3"
pub fn name_get_tag(name: &str) -> &str {
    name.split('[').next().unwrap()
}

/// Extract the generating sphere from a generator
/// Example: "5 3[6]" -> 6
pub fn name_to_sphere(name: &str) -> i32 {
    name.split(']').next().unwrap().split('[').skip(1).next().unwrap().parse().unwrap()
}


/// Extract the generating name from a generator
/// Example: "5 3[6]" -> "3[5]"
/// Takes the first number and moves it to the bracket, removes the rest
pub fn generated_by_name(name: &str) -> String {
    let initial = name.split('[').next().unwrap();

    let split_first: Vec<&str> = initial.split_whitespace().collect();
    let end = format!("[{}]", split_first.first().unwrap_or(&"0"));

    if split_first.len() <= 1 {
        end
    } else {
        format!("{}{}", split_first[1..].join(" "), end)
    }
}


/// Extract what this generator will generate
/// Example: "5 3[6]" -> "6 5 3"
/// Moves the bracket content to the front, followed by the rest
pub fn generating_tag(name: &str) -> String {
    let parts: Vec<&str> = name.split('[').collect();

    if parts.len() < 2 {
        return String::new();
    }

    let initial = parts[0];
    let real_last = parts[1].split(']').next().unwrap_or("");

    if initial.is_empty() {
        real_last.to_string()
    } else {
        format!("{} {}", real_last, initial)
    }
}
