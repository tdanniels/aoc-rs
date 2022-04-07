use std::error;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufRead, Error, ErrorKind};

static FILENAME: &str = "input.txt";

type DResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
struct Grid<T> {
    cells: Vec<T>,
    num_rows: usize,
    num_cols: usize,
}

/// Indexed by (row, col) like:
/// 0,0  0,1  0,2 ...
/// 1,0  1,1  1,2 ...
///  .    .    .
///  .    .    .
///  .    .    .
impl Grid<u32> {
    fn from_file(filename: &str) -> DResult<Self> {
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
        let cells: Vec<u32> = lines
            .iter()
            .flat_map(|s| s.chars().map(|c| c.to_digit(10).ok_or("Bad char")))
            .collect::<Result<Vec<_>, &str>>()?;
        Ok(Grid {
            cells: cells,
            num_rows: num_rows,
            num_cols: num_cols,
        })
    }

    fn at(&self, row_idx: usize, col_idx: usize) -> Option<u32> {
        if row_idx >= self.num_rows || col_idx >= self.num_cols {
            return None;
        }
        Some(self.cells[row_idx * self.num_cols + col_idx])
    }

    /// The elements of the output vector are respectively
    /// N W E S relative to the input coordinates.
    fn neighbourhood(&self, row_idx: usize, col_idx: usize) -> Option<Vec<Option<u32>>> {
        if row_idx >= self.num_rows || col_idx >= self.num_cols {
            return None;
        }
        let mut out: Vec<Option<u32>> = Vec::new();

        for (cond, (r, c)) in [
            (row_idx > 0, (row_idx.overflowing_sub(1).0, col_idx)),
            (col_idx > 0, (row_idx, col_idx.overflowing_sub(1).0)),
            (col_idx < self.num_cols - 1, (row_idx, col_idx + 1)),
            (row_idx < self.num_rows - 1, (row_idx + 1, col_idx)),
        ] {
            if cond {
                out.push(self.at(r, c));
            } else {
                out.push(None);
            }
        }
        Some(out)
    }
}

fn part1(grid: &Grid<u32>) -> DResult<u64> {
    let mut accum: u64 = 0;
    for i in 0..grid.num_rows {
        for j in 0..grid.num_cols {
            let centre = grid.at(i, j).ok_or("Bad centerpoint coords?")?;
            if grid
                .neighbourhood(i, j)
                .ok_or("Bad neighbourhood coords?")?
                .iter()
                .all(|&x| {
                    if let Some(neighbour) = x {
                        centre < neighbour
                    } else {
                        true
                    }
                })
            {
                accum += centre as u64 + 1
            }
        }
    }
    Ok(accum)
}

fn main() -> DResult<()> {
    let grid: Grid<u32> = Grid::from_file(FILENAME)?;

    println!("Part 1: {}", part1(&grid)?);

    Ok(())
}
