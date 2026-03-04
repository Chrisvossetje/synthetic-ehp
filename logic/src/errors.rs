

// pub enum Error {
//     AlgebraicNaturality(at: i32, )
// }


pub enum Error {
    AlgebraicNaturality { from: usize, to: usize }

}