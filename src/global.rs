//  rasmx86_64  - global.rs
//  -----------------------
//  made by matissoss
//  licensed under MPL 2.0

// Orb = A or B
#[derive(PartialEq, Debug, Eq, Hash, Ord, PartialOrd)]
pub enum Orb<T, Y> {
    A(T),
    B(Y)
}
