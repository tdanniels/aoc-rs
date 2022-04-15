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

    pub fn at(&self, row_idx: usize, col_idx: usize) -> Option<u8> {
        if row_idx >= self.num_rows || col_idx >= self.num_cols {
            return None;
        }
        Some(self.cells[row_idx * self.num_cols + col_idx])
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
                out.push(((r, c), self.at(r, c)));
            } else {
                out.push(((r, c), None));
            }
        }
        Some(out)
    }
}
