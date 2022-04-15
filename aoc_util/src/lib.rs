use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct AocError {
    err: String,
}

impl AocError {
    fn new(err: &str) -> Self {
        AocError {
            err: err.to_string(),
        }
    }
}

pub type AocResult<T> = std::result::Result<T, Box<dyn error::Error>>;

impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl error::Error for AocError {}

pub fn failure<T>(err: &str) -> AocResult<T> {
    Err(Box::new(AocError::new(err)))
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub i: usize,
    pub j: usize,
}

impl Point {
    pub fn new(i: usize, j: usize) -> Self {
        Point { i: i, j: j }
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

#[derive(Debug)]
pub struct Grid {
    cells: Vec<u8>,
    num_rows: usize,
    num_cols: usize,
}

/// Indexed by (row, col) like:
/// 0,0  0,1  0,2 ...
/// 1,0  1,1  1,2 ...
///  .    .    .
///  .    .    .
///  .    .    .
impl Grid {
    pub fn from_file(filename: &str) -> AocResult<Self> {
        let file = File::open(filename)?;
        let lines: Vec<String> = io::BufReader::new(file)
            .lines()
            .collect::<io::Result<_>>()?;
        let num_rows = lines.len();
        let num_cols = lines.get(0).ok_or("First row empty?")?.len();
        if !lines.iter().all(|l| l.len() == num_cols) {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Not all rows have the same number of columns.",
            )));
        }
        let cells: Vec<u8> = lines
            .iter()
            .flat_map(|s| {
                s.chars().map(|c| {
                    u8::try_from(
                        c.to_digit(10)
                            .ok_or("Bad char")
                            .map_err(|e| AocError::new(e))?,
                    )
                    .map_err(|e| AocError::new(&e.to_string()))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Grid {
            cells: cells,
            num_rows: num_rows,
            num_cols: num_cols,
        })
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    pub fn at(&self, p: Point) -> AocResult<u8> {
        if p.i >= self.num_rows || p.j >= self.num_cols {
            return failure(&format!("Invalid coordinates {}", p));
        }
        Ok(self.cells[p.i * self.num_cols + p.j])
    }

    /// The elements of the output vector are respectively
    /// N W E S relative to the input coordinates.
    pub fn neighbourhood(
        &self,
        row_idx: usize,
        col_idx: usize,
    ) -> Option<Vec<((usize, usize), Option<u8>)>> {
        if row_idx >= self.num_rows || col_idx >= self.num_cols {
            return None;
        }
        let mut out: Vec<((usize, usize), Option<u8>)> = Vec::new();

        for (cond, (r, c)) in [
            (row_idx > 0, (row_idx.overflowing_sub(1).0, col_idx)),
            (col_idx > 0, (row_idx, col_idx.overflowing_sub(1).0)),
            (col_idx < self.num_cols - 1, (row_idx, col_idx + 1)),
            (row_idx < self.num_rows - 1, (row_idx + 1, col_idx)),
        ] {
            if cond {
                out.push(((r, c), self.at(Point::new(r, c)).ok()));
            } else {
                out.push(((r, c), None));
            }
        }
        Some(out)
    }

    pub fn set(&mut self, point: Point, value: u8) -> AocResult<()> {
        if point.i >= self.num_rows || point.j >= self.num_cols {
            return failure(&format!("Invalid coordinates {}", point));
        }
        self.cells[point.i * self.num_cols + point.j] = value;
        Ok(())
    }

    // TODO: refactor neighbourhood's API to more closely resemble this one.
    /// Returns: Err(...) if `point` is an invalid coordinate (i.e., outside the grid).
    ///          Returns Ok(...) otherwise.
    /// The returned `Vec` always has 8 elements in NW, N, NE, W, E, SE, S, SE order.
    /// The elements will be `None` if they are off the grid, otherwise they will be of
    /// the form (point coordinate pair, value).
    pub fn neighbourhood8(&self, point: Point) -> AocResult<Vec<Option<(Point, u8)>>> {
        if point.i >= self.num_rows || point.j >= self.num_cols {
            return failure(&format!("Invalid coordinates {}", point));
        }
        let mut out: Vec<Option<(Point, u8)>> = Vec::new();

        let n_ok = point.i > 0;
        let w_ok = point.j > 0;
        let e_ok = point.j < self.num_cols - 1;
        let s_ok = point.i < self.num_rows - 1;

        let n_coord = point.i.overflowing_sub(1).0;
        let w_coord = point.j.overflowing_sub(1).0;
        let e_coord = point.j + 1;
        let s_coord = point.i + 1;

        for (cond, p) in [
            (n_ok && w_ok, Point::new(n_coord, w_coord)),
            (n_ok, Point::new(n_coord, point.j)),
            (n_ok && e_ok, Point::new(n_coord, e_coord)),
            (w_ok, Point::new(point.i, w_coord)),
            (e_ok, Point::new(point.i, e_coord)),
            (s_ok && w_ok, Point::new(s_coord, w_coord)),
            (s_ok, Point::new(s_coord, point.j)),
            (s_ok && e_ok, Point::new(s_coord, e_coord)),
        ] {
            if cond {
                out.push(Some((p, self.at(p)?)));
            } else {
                out.push(None);
            }
        }
        Ok(out)
    }
}
