use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::env;
use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn get_cli_arg() -> AocResult<String> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return failure(format!("Bad CLI args: {:?}", args));
    }
    Ok(args.pop().unwrap())
}

pub fn get_input_file(codefile: &str) -> AocResult<String> {
    get_data_file(codefile, "input")
}

pub fn get_test_file(codefile: &str) -> AocResult<String> {
    get_data_file(codefile, "test")
}

fn get_data_file(codefile: &str, kind: &str) -> AocResult<String> {
    let stem = Path::new(codefile)
        .file_stem()
        .ok_or(format!("No stem for {codefile}?"))?;
    let datafile = "data/".to_string()
        + stem
            .to_str()
            .ok_or(format!("OsStr {stem:?} -> str failed?"))?
        + "_"
        + kind
        + ".txt";
    Ok(datafile)
}

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

pub fn failure<T, S: AsRef<str>>(err: S) -> AocResult<T> {
    Err(Box::new(AocError::new(err.as_ref())))
}

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

#[derive(Debug)]
pub struct Grid {
    cells: Vec<u8>,
    num_rows: usize,
    num_cols: usize,
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
            cells,
            num_rows,
            num_cols,
        })
    }

    pub fn from_vec(v: Vec<u8>, num_rows: usize, num_cols: usize) -> AocResult<Self> {
        if v.len() != num_rows * num_cols {
            return failure(format!(
                "Vec len {} doesn't equal num_rows={} * num_cols={}",
                v.len(),
                num_rows,
                num_cols
            ));
        }
        Ok(Grid {
            cells: v,
            num_rows,
            num_cols,
        })
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
        if p.i >= self.num_rows || p.j >= self.num_cols {
            return failure(format!("Invalid coordinates {}", p));
        }
        Ok(self.cells[p.i * self.num_cols + p.j])
    }

    pub fn set(&mut self, point: Point, value: u8) -> AocResult<()> {
        if point.i >= self.num_rows || point.j >= self.num_cols {
            return failure(format!("Invalid coordinates {}", point));
        }
        self.cells[point.i * self.num_cols + point.j] = value;
        Ok(())
    }

    /// Returns: Err(...) if `point` is an invalid coordinate (i.e., outside the grid).
    ///          Returns Ok(...) otherwise.
    /// The returned `Vec`'s elements and ordering are chosen according to NeighbourPattern.
    /// The elements will be `None` if they are off the grid, otherwise they will be of
    /// the form (point coordinate pair, value).
    pub fn neighbourhood(
        &self,
        point: Point,
        neighbour_pattern: NeighbourPattern,
    ) -> AocResult<Vec<Option<(Point, u8)>>> {
        if point.i >= self.num_rows || point.j >= self.num_cols {
            return failure(format!("Invalid coordinates {}", point));
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
        if point.i >= self.num_rows || point.j >= self.num_cols {
            return failure(format!("Invalid coordinates {}", point));
        }
        Ok(self.num_cols * point.i + point.j)
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

        while q.len() != 0 {
            let u_index = q.pop().unwrap().0.idx;
            let u_point = self.point_from_index(u_index)?;
            for v in self.neighbourhood(u_point, neighbour_pattern)? {
                if let Some(v) = v {
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
                        if q.iter().find(|&x| x.0.idx == v_index).is_none() {
                            q.push(Reverse(DistIdx {
                                dist: alt,
                                idx: v_index,
                            }));
                        }
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

/// Represents a graph as a vector of named nodes, and a set of pairs of indices into
/// that vector which represents its edges. The node2index member maps from node names
/// to their indices.
#[derive(Debug)]
pub struct UnweightedUndirectedGraph {
    nodes: Vec<String>,
    edges: HashSet<(usize, usize)>,
    node2index: HashMap<String, usize>,
}

impl UnweightedUndirectedGraph {
    pub fn from_file(filename: &str) -> AocResult<Self> {
        let mut nodes: Vec<String> = Vec::new();
        let mut edges: HashSet<(usize, usize)> = HashSet::new();
        let mut node2index: HashMap<String, usize> = HashMap::new();

        let file = File::open(filename)?;
        for line in io::BufReader::new(file).lines() {
            let edge = line?.split('-').map(String::from).collect::<Vec<String>>();
            if edge.len() != 2
                || !edge
                    .iter()
                    .all(|v| v.chars().all(|c| c.is_ascii_alphabetic()))
            {
                return failure(format!("Malformed edge {:?} in input", edge));
            }

            for i in 0..2 {
                if node2index.get(&edge[i]).is_none() {
                    nodes.push(edge[i].clone());
                    node2index.insert(nodes[nodes.len() - 1].clone(), nodes.len() - 1);
                }
            }
            edges.insert((
                *node2index.get(&edge[0]).unwrap(),
                *node2index.get(&edge[1]).unwrap(),
            ));
        }
        Ok(UnweightedUndirectedGraph {
            nodes,
            edges,
            node2index,
        })
    }

    pub fn index(&self, node: &str) -> AocResult<usize> {
        Ok(self
            .node2index
            .get(node)
            .ok_or(format!("No such node {}", node))
            .map(|x| *x)?)
    }

    pub fn neighbours(&self, node: &str) -> AocResult<Vec<&str>> {
        let index = self.index(node)?;
        Ok(self
            .edges
            .iter()
            .filter(|e| e.0 == index || e.1 == index)
            .map(|e| {
                if e.0 == index {
                    self.nodes[e.1].as_str()
                } else {
                    self.nodes[e.0].as_str()
                }
            })
            .collect())
    }
}
