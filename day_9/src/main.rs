use std::collections::{BinaryHeap, HashSet, VecDeque};
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
    fn neighbourhood(
        &self,
        row_idx: usize,
        col_idx: usize,
    ) -> Option<Vec<((usize, usize), Option<u32>)>> {
        if row_idx >= self.num_rows || col_idx >= self.num_cols {
            return None;
        }
        let mut out: Vec<((usize, usize), Option<u32>)> = Vec::new();

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

    fn find_low_points(&self) -> Vec<((usize, usize), u64)> {
        let mut out = Vec::new();
        for i in 0..self.num_rows {
            for j in 0..self.num_cols {
                let centre = self.at(i, j).expect("Bad centrepoint coords?");
                if self
                    .neighbourhood(i, j)
                    .expect("Bad neighbourhood coords?")
                    .iter()
                    .all(|&x| {
                        if let Some(neighbour_height) = x.1 {
                            centre < neighbour_height
                        } else {
                            true
                        }
                    })
                {
                    out.push(((i, j), centre as u64));
                }
            }
        }
        out
    }
}

/// Assumes that starting_point is a low point. Should fix this implicit assumption.
fn get_basin_size(grid: &Grid<u32>, starting_point: &(usize, usize)) -> u64 {
    let mut q: VecDeque<(usize, usize)> = VecDeque::new();
    let mut explored: HashSet<(usize, usize)> = HashSet::new();
    explored.insert(*starting_point);
    q.push_back(*starting_point);
    while q.len() > 0 {
        let v = q.pop_front().unwrap();
        for neighbour in grid.neighbourhood(v.0, v.1).unwrap() {
            if !neighbour.1.is_some() {
                continue;
            }
            let neighbour_height = neighbour.1.unwrap();
            if neighbour_height <= grid.at(v.0, v.1).unwrap() || neighbour_height == 9 {
                continue;
            }
            if explored.get(&neighbour.0).is_none() {
                explored.insert(neighbour.0);
                q.push_back(neighbour.0);
            }
        }
    }
    explored.len() as u64
}

fn part1(grid: &Grid<u32>) -> DResult<u64> {
    let mut accum: u64 = 0;
    for p in grid.find_low_points() {
        accum += p.1 as u64 + 1
    }
    Ok(accum)
}

fn part2(grid: &Grid<u32>) -> DResult<u64> {
    let low_points = grid.find_low_points();

    Ok(low_points
        .iter()
        .map(|x| get_basin_size(grid, &x.0))
        .collect::<BinaryHeap<u64>>()
        .into_sorted_vec()
        .iter()
        .rev()
        .take(3)
        .product())
}

fn main() -> DResult<()> {
    let grid: Grid<u32> = Grid::from_file(FILENAME)?;

    println!("Part 1: {}", part1(&grid)?);
    println!("Part 2: {}", part2(&grid)?);

    Ok(())
}
