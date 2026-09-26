// use crate::types::Torsion;

// pub fn diff_sorted(
//     a: &[Torsion],
//     b: &[Torsion],
// ) -> (Vec<Torsion>, Vec<Torsion>) {
//     let mut only_a = Vec::new();
//     let mut only_b = Vec::new();

//     let mut i = 0;
//     let mut j = 0;

//     while i < a.len() && j < b.len() {
//         match a[i].cmp(&b[j]) {
//             std::cmp::Ordering::Less => {
//                 only_a.push(a[i].clone());
//                 i += 1;
//             }
//             std::cmp::Ordering::Greater => {
//                 only_b.push(b[j].clone());
//                 j += 1;
//             }
//             std::cmp::Ordering::Equal => {
//                 i += 1;
//                 j += 1;
//             }
//         }
//     }

//     // remaining tail
//     only_a.extend_from_slice(&a[i..]);
//     only_b.extend_from_slice(&b[j..]);

//     (only_a, only_b)
// }