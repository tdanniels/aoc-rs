use crate::errors::{failure, AocError, AocResult};
use crate::point::Point;

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grid {
    cells: Vec<u8>,
    num_rows: usize,
    num_cols: usize,
    is_toroidal: bool,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for i in 0..self.num_rows {
            for j in 0..self.num_cols {
                s += self.cells[i * self.num_cols + j].to_string().as_str();
                if j == self.num_cols - 1 && i != self.num_rows - 1 {
                    s += "\n";
                }
            }
        }
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NeighbourPattern {
    /// N W E S
    Compass4,
    /// NW N NE W E SW S SE
    Compass8,
}

/// Indexed by (row, col) like:
/// 0,0  0,1  0,2 ...
/// 1,0  1,1  1,2 ...
///  .    .    .
///  .    .    .
///  .    .    .
impl Grid {
    // TODO: update to use a an iterable of AsRef<str> instead of `filename`.
    pub fn from_digit_matrix_file(filename: &str) -> AocResult<Self> {
        let file = File::open(filename)?;
        let lines: Vec<String> = io::BufReader::new(file)
            .lines()
            .collect::<io::Result<_>>()?;
        let num_rows = lines.len();
        let num_cols = lines.get(0).ok_or("First row empty?")?.len();
        if !lines.iter().all(|l| l.len() == num_cols) {
            return failure("Not all rows have the same number of columns.");
        }
        let cells: Vec<u8> = lines
            .iter()
            .flat_map(|s| {
                s.chars().map(|c| {
                    u8::try_from(c.to_digit(10).ok_or("Bad char").map_err(AocError::new)?)
                        .map_err(|e| AocError::new(e.to_string()))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Grid {
            cells,
            num_rows,
            num_cols,
            is_toroidal: false,
        })
    }

    // TODO: update to use a an iterable of AsRef<str> instead of &[String].
    pub fn from_symbol_matrix<F>(lines: &[String], map_func: F) -> AocResult<Self>
    where
        F: Fn(char) -> Option<u8>,
    {
        let num_rows = lines.len();
        let num_cols = lines.get(0).ok_or("First row empty?")?.len();
        if !lines.iter().all(|l| l.len() == num_cols) {
            return failure("Not all rows have the same number of columns.");
        }
        let cells: Vec<u8> = lines
            .iter()
            .flat_map(|s| {
                s.chars()
                    .map(|c| map_func(c).ok_or(format!("Bad char {c}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Grid {
            cells,
            num_rows,
            num_cols,
            is_toroidal: false,
        })
    }

    pub fn from_slice(slice: &[u8], num_rows: usize, num_cols: usize) -> AocResult<Self> {
        if slice.len() != num_rows * num_cols {
            return failure(format!(
                "Vec len {} doesn't equal num_rows={} * num_cols={}",
                slice.len(),
                num_rows,
                num_cols
            ));
        }
        Ok(Grid {
            cells: slice.to_vec(),
            num_rows,
            num_cols,
            is_toroidal: false,
        })
    }

    /// Treats points outside the grid as if they loop around instead
    /// of being invalid. Note that it's currently only possible to loop around
    /// from the bottom of the grid to the top, and from the right to the left,
    /// since grid coordinates are unsigned.
    pub fn make_toroidal(&mut self, is_toroidal: bool) {
        self.is_toroidal = is_toroidal;
    }

    pub fn is_toroidal(&self) -> bool {
        self.is_toroidal
    }

    pub fn vec(&self) -> &Vec<u8> {
        &self.cells
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    pub fn at(&self, p: Point) -> AocResult<u8> {
        if !self.is_toroidal && (p.i >= self.num_rows || p.j >= self.num_cols) {
            return failure(format!("Invalid coordinates {}", p));
        }
        Ok(self.cells[(p.i % self.num_rows) * self.num_cols + (p.j % self.num_cols)])
    }

    pub fn set(&mut self, point: Point, value: u8) -> AocResult<()> {
        if !self.is_toroidal && (point.i >= self.num_rows || point.j >= self.num_cols) {
            return failure(format!("Invalid coordinates {}", point));
        }
        self.cells[(point.i % self.num_rows) * self.num_cols + (point.j % self.num_cols)] =
            value;
        Ok(())
    }

    /// Returns: Err(...) if `point` is an invalid coordinate (i.e., outside the grid) and
    ///          the grid is not toroidal.
    ///          Returns Ok(...) otherwise.
    /// The returned `Vec`'s elements and ordering are chosen according to NeighbourPattern.
    /// The elements will be `None` if they are off the grid (and the grid is not toroidal),
    /// otherwise they will be of the form (point coordinate pair, value).
    pub fn neighbourhood(
        &self,
        point: Point,
        neighbour_pattern: NeighbourPattern,
    ) -> AocResult<Vec<Option<(Point, u8)>>> {
        if !self.is_toroidal && (point.i >= self.num_rows || point.j >= self.num_cols) {
            return failure(format!("Invalid coordinates {}", point));
        }
        let mut out: Vec<Option<(Point, u8)>> = Vec::new();

        let point = Point::new(point.i % self.num_rows, point.j % self.num_cols);

        let n_ok = self.is_toroidal || (point.i > 0);
        let w_ok = self.is_toroidal || (point.j > 0);
        let e_ok = self.is_toroidal || (point.j < self.num_cols - 1);
        let s_ok = self.is_toroidal || (point.i < self.num_rows - 1);

        let n_coord = if let Some(v) = point.i.checked_sub(1) {
            v
        } else {
            self.num_rows - 1
        };
        let w_coord = if let Some(v) = point.j.checked_sub(1) {
            v
        } else {
            self.num_cols - 1
        };
        let e_coord = (point.j + 1) % self.num_cols;
        let s_coord = (point.i + 1) % self.num_rows;

        let conditions: Vec<(bool, Point)> = match neighbour_pattern {
            NeighbourPattern::Compass4 => vec![
                (n_ok, Point::new(n_coord, point.j)),
                (w_ok, Point::new(point.i, w_coord)),
                (e_ok, Point::new(point.i, e_coord)),
                (s_ok, Point::new(s_coord, point.j)),
            ],
            NeighbourPattern::Compass8 => vec![
                (n_ok && w_ok, Point::new(n_coord, w_coord)),
                (n_ok, Point::new(n_coord, point.j)),
                (n_ok && e_ok, Point::new(n_coord, e_coord)),
                (w_ok, Point::new(point.i, w_coord)),
                (e_ok, Point::new(point.i, e_coord)),
                (s_ok && w_ok, Point::new(s_coord, w_coord)),
                (s_ok, Point::new(s_coord, point.j)),
                (s_ok && e_ok, Point::new(s_coord, e_coord)),
            ],
        };

        for (cond, p) in conditions {
            if cond {
                out.push(Some((p, self.at(p)?)));
            } else {
                out.push(None);
            }
        }
        Ok(out)
    }

    fn point_from_index(&self, index: usize) -> AocResult<Point> {
        if index >= self.num_rows * self.num_cols {
            return failure(format!("Invalid index {index}"));
        }
        Ok(Point::new(index / self.num_rows, index % self.num_cols))
    }

    fn index_from_point(&self, point: Point) -> AocResult<usize> {
        if !self.is_toroidal && (point.i >= self.num_rows || point.j >= self.num_cols) {
            return failure(format!("Invalid coordinates {}", point));
        }
        Ok(self.num_cols * (point.i % self.num_rows) + (point.j % self.num_cols))
    }

    pub fn dijkstra(
        &self,
        start: Point,
        finish: Point,
        neighbour_pattern: NeighbourPattern,
    ) -> AocResult<(Vec<Point>, Option<u64>)> {
        let mut dist: Vec<Option<u64>> = vec![None; self.num_rows * self.num_cols];
        let mut prev: Vec<Option<usize>> = vec![None; self.num_rows * self.num_cols];
        let mut q: BinaryHeap<Reverse<DistIdx>> = BinaryHeap::new();
        let start_index = self.index_from_point(start)?;
        let finish_index = self.index_from_point(finish)?;

        dist[start_index] = Some(0);
        q.push(Reverse(DistIdx {
            dist: dist[start_index].unwrap(),
            idx: start_index,
        }));

        while !q.is_empty() {
            let u_index = q.pop().unwrap().0.idx;
            let u_point = self.point_from_index(u_index)?;
            for v in self
                .neighbourhood(u_point, neighbour_pattern)?
                .into_iter()
                .flatten()
            {
                let v_index = self.index_from_point(v.0)?;
                let alt = {
                    if let Some(d) = dist[u_index] {
                        d + v.1 as u64
                    } else {
                        u64::MAX
                    }
                };

                if alt < dist[v_index].map_or(u64::MAX, |x| x) {
                    dist[v_index] = Some(alt);
                    prev[v_index] = Some(u_index);
                    if !q.iter().any(|x| x.0.idx == v_index) {
                        q.push(Reverse(DistIdx {
                            dist: alt,
                            idx: v_index,
                        }));
                    }
                }
            }
        }

        // Construct the shortest path Vec
        let mut out: VecDeque<Point> = VecDeque::new();
        let mut u_index = Some(finish_index);
        if prev[u_index.unwrap()].is_some() || u_index.unwrap() == start_index {
            while u_index.is_some() {
                out.push_front(self.point_from_index(u_index.unwrap())?);
                u_index = prev[u_index.unwrap()];
            }
        }

        Ok((out.drain(..).collect(), dist[finish_index]))
    }

    pub fn add_border(&mut self, border_size: usize, border_fill: u8) {
        if border_size == 0 {
            return;
        }
        let new_len = (self.num_rows + border_size * 2) * (self.num_cols + border_size * 2);
        let mut new_cells = Vec::with_capacity(new_len);
        new_cells.resize(new_len, border_fill);
        let mut new_grid = Grid::from_slice(
            new_cells.as_slice(),
            self.num_rows + border_size * 2,
            self.num_cols + border_size * 2,
        )
        .unwrap();
        new_grid.is_toroidal = self.is_toroidal;
        for i in 0..self.num_rows() {
            for j in 0..self.num_cols() {
                let p_old = Point::new(i, j);
                let p_new = Point::new(border_size + i, border_size + j);
                new_grid.set(p_new, self.at(p_old).unwrap()).unwrap();
            }
        }
        *self = new_grid;
    }
}

#[derive(Eq)]
struct DistIdx {
    dist: u64,
    idx: usize,
}

impl Ord for DistIdx {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist.cmp(&other.dist)
    }
}

impl PartialOrd for DistIdx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DistIdx {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn grid_border() -> AocResult<()> {
        #[rustfmt::skip]
        let mut grid = Grid::from_slice(&[
            1, 2, 3,
            4, 5, 6], 2, 3)?;
        grid.add_border(2, 9);
        #[rustfmt::skip]
        let mut grid2 = Grid::from_slice(&[
            9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9,
            9, 9, 1, 2, 3, 9, 9,
            9, 9, 4, 5, 6, 9, 9,
            9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9,
        ], 6, 7)?;
        assert_eq!(grid, grid2);
        grid2.add_border(1, 0);
        #[rustfmt::skip]
        let grid3 = Grid::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 9, 9, 9, 9, 9, 9, 9, 0,
            0, 9, 9, 9, 9, 9, 9, 9, 0,
            0, 9, 9, 1, 2, 3, 9, 9, 0,
            0, 9, 9, 4, 5, 6, 9, 9, 0,
            0, 9, 9, 9, 9, 9, 9, 9, 0,
            0, 9, 9, 9, 9, 9, 9, 9, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ], 8, 9)?;
        assert_eq!(grid2, grid3);
        Ok(())
    }

    #[test]
    fn at() -> AocResult<()> {
        #[rustfmt::skip]
        let mut grid = Grid::from_slice(&[
            1, 2, 3,
            4, 5, 6], 2, 3)?;

        assert_eq!(grid.at(Point::new(0, 0))?, 1);
        assert_eq!(grid.at(Point::new(0, 1))?, 2);
        assert_eq!(grid.at(Point::new(0, 2))?, 3);
        assert_eq!(grid.at(Point::new(1, 0))?, 4);
        assert_eq!(grid.at(Point::new(1, 1))?, 5);
        assert_eq!(grid.at(Point::new(1, 2))?, 6);

        grid.make_toroidal(true);

        assert_eq!(grid.at(Point::new(0, 0))?, 1);
        assert_eq!(grid.at(Point::new(0, 1))?, 2);
        assert_eq!(grid.at(Point::new(0, 2))?, 3);
        assert_eq!(grid.at(Point::new(1, 0))?, 4);
        assert_eq!(grid.at(Point::new(1, 1))?, 5);
        assert_eq!(grid.at(Point::new(1, 2))?, 6);

        assert_eq!(grid.at(Point::new(2, 0))?, 1);
        assert_eq!(grid.at(Point::new(2, 1))?, 2);
        assert_eq!(grid.at(Point::new(2, 2))?, 3);
        assert_eq!(grid.at(Point::new(3, 0))?, 4);
        assert_eq!(grid.at(Point::new(3, 1))?, 5);
        assert_eq!(grid.at(Point::new(3, 2))?, 6);

        assert_eq!(grid.at(Point::new(0, 3))?, 1);
        assert_eq!(grid.at(Point::new(0, 4))?, 2);
        assert_eq!(grid.at(Point::new(0, 5))?, 3);
        assert_eq!(grid.at(Point::new(1, 3))?, 4);
        assert_eq!(grid.at(Point::new(1, 4))?, 5);
        assert_eq!(grid.at(Point::new(1, 5))?, 6);

        assert_eq!(grid.at(Point::new(2, 3))?, 1);
        assert_eq!(grid.at(Point::new(2, 4))?, 2);
        assert_eq!(grid.at(Point::new(2, 5))?, 3);
        assert_eq!(grid.at(Point::new(3, 3))?, 4);
        assert_eq!(grid.at(Point::new(3, 4))?, 5);
        assert_eq!(grid.at(Point::new(3, 5))?, 6);

        Ok(())
    }

    #[test]
    fn neighbours() -> AocResult<()> {
        #[rustfmt::skip]
        let mut grid = Grid::from_slice(&[
            1, 2, 3,
            4, 5, 6], 2, 3)?;
        assert_eq!(
            grid.neighbourhood(Point::new(0, 0), NeighbourPattern::Compass4)?,
            vec![
                None,
                None,
                Some((Point::new(0, 1), 2)),
                Some((Point::new(1, 0), 4))
            ]
        );
        assert_eq!(
            grid.neighbourhood(Point::new(0, 0), NeighbourPattern::Compass8)?,
            vec![
                None,
                None,
                None,
                None,
                Some((Point::new(0, 1), 2)),
                None,
                Some((Point::new(1, 0), 4)),
                Some((Point::new(1, 1), 5))
            ]
        );

        grid.make_toroidal(true);
        assert_eq!(
            grid.neighbourhood(Point::new(0, 0), NeighbourPattern::Compass4)?,
            vec![
                Some((Point::new(1, 0), 4)),
                Some((Point::new(0, 2), 3)),
                Some((Point::new(0, 1), 2)),
                Some((Point::new(1, 0), 4))
            ]
        );
        assert_eq!(
            grid.neighbourhood(Point::new(0, 0), NeighbourPattern::Compass8)?,
            vec![
                Some((Point::new(1, 2), 6)),
                Some((Point::new(1, 0), 4)),
                Some((Point::new(1, 1), 5)),
                Some((Point::new(0, 2), 3)),
                Some((Point::new(0, 1), 2)),
                Some((Point::new(1, 2), 6)),
                Some((Point::new(1, 0), 4)),
                Some((Point::new(1, 1), 5))
            ]
        );
        Ok(())
    }
}
