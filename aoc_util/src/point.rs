use std::fmt;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Point {
    pub i: usize,
    pub j: usize,
}

impl Point {
    pub fn new(i: usize, j: usize) -> Self {
        Point { i, j }
    }
    pub fn from_pair(pair: (usize, usize)) -> Self {
        Point {
            i: pair.0,
            j: pair.1,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.i, self.j)
    }
}
