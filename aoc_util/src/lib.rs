use std::cell::RefCell;
use std::cmp::{max, min, Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::env;
use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::slice::Iter;
use std::str::FromStr;

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
    pub fn new<S: AsRef<str>>(err: S) -> Self {
        AocError {
            err: err.as_ref().to_string(),
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
    // TODO: update to use a an iterable of String instead of `filename`.
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

pub type NodeLink = Rc<RefCell<Node>>;

#[derive(Clone, Debug)]
pub struct Node {
    data: Option<i64>,
    left: Option<NodeLink>,
    right: Option<NodeLink>,
    parent: Option<Weak<RefCell<Node>>>,
}

impl Node {
    pub fn new(data: Option<i64>) -> NodeLink {
        Rc::new(RefCell::new(Node {
            data,
            left: None,
            right: None,
            parent: None,
        }))
    }

    pub fn new_with_parent(data: Option<i64>, parent: &NodeLink) -> NodeLink {
        Rc::new(RefCell::new(Node {
            data,
            left: None,
            right: None,
            parent: Some(Rc::downgrade(parent)),
        }))
    }
}

#[derive(Clone, Debug)]
pub struct NodeWrapper(NodeLink);

impl From<NodeLink> for NodeWrapper {
    fn from(n: NodeLink) -> NodeWrapper {
        NodeWrapper(n)
    }
}

impl NodeWrapper {
    pub fn get_left(&self) -> Option<NodeWrapper> {
        if let Some(left) = &self.0.borrow().left {
            Some(left.clone().into())
        } else {
            None
        }
    }

    pub fn get_right(&self) -> Option<NodeWrapper> {
        if let Some(right) = &self.0.borrow().right {
            Some(right.clone().into())
        } else {
            None
        }
    }

    pub fn get_data(&self) -> Option<i64> {
        self.0.borrow().data
    }

    pub fn get_parent(&self) -> Option<NodeWrapper> {
        if let Some(parent) = &self.0.borrow().parent {
            Some(parent.upgrade().unwrap().into())
        } else {
            None
        }
    }

    pub fn set_left(&self, child: Option<&NodeWrapper>) {
        if let Some(child) = child {
            self.0.borrow_mut().left = Some(child.0.clone());
            child.0.borrow_mut().parent = Some(Rc::downgrade(&self.0));
        } else {
            self.0.borrow_mut().left = None
        }
    }

    pub fn set_right(&self, child: Option<&NodeWrapper>) {
        if let Some(child) = child {
            self.0.borrow_mut().right = Some(child.0.clone());
            child.0.borrow_mut().parent = Some(Rc::downgrade(&self.0));
        } else {
            self.0.borrow_mut().right = None
        }
    }

    pub fn set_data(&self, data: Option<i64>) {
        self.0.borrow_mut().data = data;
    }

    pub fn is_leaf(&self) -> bool {
        self.get_left().is_none() && self.get_right().is_none()
    }

    pub fn has_data(&self) -> bool {
        self.get_data().is_some()
    }

    pub fn depth_first_iter(&self) -> DepthFirstIterator {
        DepthFirstIterator::new(&self.0)
    }

    pub fn from_ascii(ascii: &[u8]) -> AocResult<NodeWrapper> {
        Ok(NodeWrapper::from(NodeWrapper::_from_ascii(ascii)?.0))
    }

    pub fn inner(&self) -> NodeLink {
        self.0.clone()
    }

    /// Parses a NodeLink from a line of ASCII of the form:
    /// "[[1,2],[3,[4,5]]]" etc.
    /// Current limitations: no whitespace, only single digit numbers supported.
    fn _from_ascii(ascii: &[u8]) -> AocResult<(NodeWrapper, usize)> {
        if ascii[0] != b'[' {
            return failure(format!("Invalid line start"));
        }

        let mut consumed = 0;
        let mut seen_comma = false;
        let mut seen_opening_bracket = false;
        let mut pair = Vec::new();

        // Another implicit state machine :(.
        loop {
            let c = ascii[consumed];
            match c {
                b'[' => {
                    if seen_opening_bracket {
                        let (node, cons) = NodeWrapper::_from_ascii(&ascii[consumed..])?;
                        consumed += cons;
                        pair.push(node);
                    } else {
                        seen_opening_bracket = true;
                        consumed += 1;
                    }
                }
                b'0'..=b'9' => {
                    if (!seen_comma && pair.len() != 0) || (seen_comma && pair.len() == 0) {
                        return failure("Invalid digit location");
                    }
                    pair.push(Node::new(Some((c - 48) as i64)).into());
                    consumed += 1;
                }
                b',' => {
                    if seen_comma {
                        return failure("Two commas in a node");
                    }
                    seen_comma = true;
                    consumed += 1;
                }
                b']' => {
                    if !seen_comma {
                        return failure("No comma in a node");
                    }
                    if pair.len() != 2 {
                        return failure(format!("Invalid 'pair': {:?}", pair));
                    }
                    consumed += 1;
                    let node = NodeWrapper::from(Node::new(None));
                    node.set_left(Some(&pair.remove(0).into()));
                    node.set_right(Some(&pair.remove(0).into()));
                    return Ok((node, consumed));
                }
                _ => return failure("Invalid character"),
            }
        }
    }

    pub fn to_string(&self) -> String {
        // TODO currently only supports trees with (required) data at leaves.
        if self.is_leaf() && !self.has_data() {
            panic!("Invalid tree: leaf with no data");
        }
        if !self.is_leaf() && self.has_data() {
            panic!("Invalid tree: non-leaf with data");
        }
        if let Some(data) = self.get_data() {
            data.to_string()
        } else {
            let left_string = NodeWrapper::from(self.get_left().unwrap()).to_string();
            let right_string = NodeWrapper::from(self.get_right().unwrap()).to_string();
            "[".to_string() + left_string.as_str() + "," + right_string.as_str() + "]"
        }
    }
}

pub struct DepthFirstIterator {
    stack: Vec<(NodeLink, usize)>,
}

impl DepthFirstIterator {
    pub fn new(node: &NodeLink) -> Self {
        let stack = vec![(node.clone(), 0)];
        DepthFirstIterator { stack }
    }
}

impl Iterator for DepthFirstIterator {
    type Item = (NodeWrapper, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.len() != 0 {
            let (node, depth) = self.stack.pop().unwrap();

            // Push right first so that we pop left first.
            if let Some(right) = node.borrow().right.clone() {
                self.stack.push((right, depth + 1));
            };
            if let Some(left) = node.borrow().left.clone() {
                self.stack.push((left, depth + 1));
            }
            return Some((node.into(), depth));
        }
        None
    }
}

#[cfg(test)]
mod nodewrapper_tests {
    use super::*;

    #[test]
    fn nodewrapper_from_ascii() -> AocResult<()> {
        for s in [
            "[1,2]",
            "[[1,2],3]",
            "[1,[2,3]]",
            "[[1,2],[3,4]]",
            "[[[[[1,2],3],[4,5]],6],[7,[[8,9],0]]]",
        ] {
            let t = NodeWrapper::from_ascii(s.as_bytes())?;
            assert_eq!(s.to_string(), t.to_string());
        }
        Ok(())
    }

    #[test]
    fn nodewrapper_depth_first_traversal() -> AocResult<()> {
        for (s, v, d) in [
            ("[1,2]", vec![1, 2], vec![1, 1]),
            ("[[1,2],3]", vec![1, 2, 3], vec![2, 2, 1]),
            ("[1,[2,3]]", vec![1, 2, 3], vec![1, 2, 2]),
            ("[[1,2],[3,4]]", vec![1, 2, 3, 4], vec![2, 2, 2, 2]),
            (
                "[[[[[1,2],3],[4,5]],6],[7,[[8,9],0]]]",
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
                vec![5, 5, 4, 4, 4, 2, 2, 4, 4, 3],
            ),
        ] {
            let t = NodeWrapper::from_ascii(s.as_bytes())?;
            let data = t
                .depth_first_iter()
                .filter_map(|(node, _depth)| node.get_data())
                .collect::<Vec<_>>();
            let depths = t
                .depth_first_iter()
                .filter_map(|(node, depth)| {
                    if let Some(_) = node.get_data() {
                        Some(depth)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(data, v);
            assert_eq!(depths, d);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq)]
pub struct Cuboid {
    x0: i64,
    x1: i64,
    y0: i64,
    y1: i64,
    z0: i64,
    z1: i64,
}

/// Accepts strings like "x=23..99,y=-100..-50,z=-1000..77"
impl FromStr for Cuboid {
    type Err = Box<dyn error::Error>;

    fn from_str(s: &str) -> AocResult<Self> {
        let (mut x0, mut x1, mut y0, mut y1, mut z0, mut z1) = (0, 0, 0, 0, 0, 0);

        for (prefix, c0, c1, has_suffix) in [
            ("x=", &mut x0, &mut x1, true),
            ("y=", &mut y0, &mut y1, true),
            ("z=", &mut z0, &mut z1, false),
        ] {
            let start =
                s.find(prefix).ok_or(format!("No prefix \"{}\"?", prefix))? + prefix.len();
            let end = if has_suffix {
                start + s[start..].find(",").ok_or("No suffix \",\"?")?
            } else {
                s.len()
            };
            let slice = &s[start..end];
            let c0_c1: Vec<i64> = slice
                .split("..")
                .map(|s| s.parse::<i64>())
                .collect::<Result<_, ParseIntError>>()?;
            if c0_c1.len() != 2 {
                return failure("Bad pair length");
            }
            *c0 = c0_c1[0];
            *c1 = c0_c1[1];
        }

        Cuboid::new(x0, x1, y0, y1, z0, z1)
    }
}

impl fmt::Display for Cuboid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {}, {}, {})",
            self.x0, self.x1, self.y0, self.y1, self.z0, self.z1
        )
    }
}

impl Cuboid {
    pub fn new(x0: i64, x1: i64, y0: i64, y1: i64, z0: i64, z1: i64) -> AocResult<Self> {
        if x0 > x1 || y0 > y1 || z0 > z1 {
            return failure("Invalid cuboid: require coord0 <= coord1");
        }
        Ok(Self {
            x0,
            x1,
            y0,
            y1,
            z0,
            z1,
        })
    }
    pub fn contains(&self, other: &Cuboid) -> bool {
        self.x0 <= other.x0
            && self.x1 >= other.x1
            && self.y0 <= other.y0
            && self.y1 >= other.y1
            && self.z0 <= other.z0
            && self.z1 >= other.z1
    }

    pub fn union(&self, other: &Cuboid) -> Vec<Cuboid> {
        if self.contains(other) {
            vec![self.clone()]
        } else if other.contains(&self) {
            vec![other.clone()]
        } else if let Some(_intersection) = self.intersection(other) {
            let mut out = vec![self.clone()];
            out.append(&mut other.difference(self));
            out
        } else {
            vec![self.clone(), other.clone()]
        }
    }

    pub fn get_coord(&self, i: i64) -> i64 {
        match i {
            0 => self.x0,
            1 => self.x1,
            2 => self.y0,
            3 => self.y1,
            4 => self.z0,
            5 => self.z1,
            _ => panic!("Invalid coordinate {i}"),
        }
    }

    pub fn set_coord(&mut self, i: i64, value: i64) {
        match i {
            0 => self.x0 = value,
            1 => self.x1 = value,
            2 => self.y0 = value,
            3 => self.y1 = value,
            4 => self.z0 = value,
            5 => self.z1 = value,
            _ => panic!("Bad coordinate index {i}"),
        }
    }

    /// Extend `self` to `other` in at most 26 different ways. Extensions
    /// are disjoint from `self` and from each other.
    pub fn extensions(&self, other: &Cuboid) -> Vec<Cuboid> {
        let mut out = Vec::with_capacity(26);
        #[rustfmt::skip]
        let a = [
            /* FA: X+, Y+, X-, Y-, Z+, Z- */
            (self.x1 + 1, other.x1, self.y0, self.y1, self.z0, self.z1),
            (self.x0, self.x1, self.y1 + 1, other.y1, self.z0, self.z1),
            (other.x0, self.x0 - 1, self.y0, self.y1, self.z0, self.z1),
            (self.x0, self.x1, other.y0, self.y0 - 1, self.z0, self.z1),
            (self.x0, self.x1, self.y0, self.y1, self.z1 + 1, other.z1),
            (self.x0, self.x1, self.y0, self.y1, other.z0, self.z0 - 1),
            /* AA Above */
            (self.x1 + 1, other.x1, self.y0, self.y1, self.z1 + 1, other.z1),
            (self.x0, self.x1, self.y1 + 1, other.y1, self.z1 + 1, other.z1),
            (other.x0, self.x0 - 1, self.y0, self.y1, self.z1 + 1, other.z1),
            (self.x0, self.x1, other.y0, self.y0 - 1, self.z1 + 1, other.z1),
            /* AA Below */
            (self.x1 + 1, other.x1, self.y0, self.y1, other.z0, self.z0 - 1),
            (self.x0, self.x1, self.y1 + 1, other.y1, other.z0, self.z0 - 1),
            (other.x0, self.x0 - 1, self.y0, self.y1, other.z0, self.z0 - 1),
            (self.x0, self.x1, other.y0, self.y0 - 1, other.z0, self.z0 - 1),
            /* Corners */
            (self.x1 + 1, other.x1, self.y1 + 1, other.y1, self.z1 + 1, other.z1),
            (other.x0, self.x0 - 1, self.y1 + 1, other.y1, self.z1 + 1, other.z1),
            (other.x0, self.x0 - 1, other.y0, self.y0 - 1, self.z1 + 1, other.z1),
            (self.x1 + 1, other.x1, other.y0, self.y0 - 1, self.z1 + 1, other.z1),
            (self.x1 + 1, other.x1, self.y1 + 1, other.y1, self.z0, self.z1),
            (other.x0, self.x0 - 1, self.y1 + 1, other.y1, self.z0, self.z1),
            (other.x0, self.x0 - 1, other.y0, self.y0 - 1, self.z0, self.z1),
            (self.x1 + 1, other.x1, other.y0, self.y0 - 1, self.z0, self.z1),
            (self.x1 + 1, other.x1, self.y1 + 1, other.y1, other.z0, self.z0 - 1),
            (other.x0, self.x0 - 1, self.y1 + 1, other.y1, other.z0, self.z0 - 1),
            (other.x0, self.x0 - 1, other.y0, self.y0 - 1, other.z0, self.z0 - 1),
            (self.x1 + 1, other.x1, other.y0, self.y0 - 1, other.z0, self.z0 - 1),
        ];
        for co in a {
            if !(co.0 > other.x1
                || co.1 < other.x0
                || co.2 > other.y1
                || co.3 < other.y0
                || co.4 > other.z1
                || co.5 < other.z0)
            {
                out.push(Cuboid::new(co.0, co.1, co.2, co.3, co.4, co.5).unwrap());
            }
        }
        debug_assert!(out.iter().all(|c| c.intersection(&self).is_none()));
        debug_assert!(out.iter().enumerate().all(|(i, c1)| out
            .iter()
            .enumerate()
            .all(|(j, c2)| i == j || c1.intersection(c2).is_none())));
        out
    }

    pub fn difference(&self, other: &Cuboid) -> Vec<Cuboid> {
        if other.contains(self) {
            vec![]
        } else if let Some(intersection) = self.intersection(other) {
            let mut out = Vec::new();
            // Extend `intersection` in all 26 possible directions, and take the
            // intersection of `ext` and `self` to obtain a possible partial difference
            // cuboid. If the new intersection is empty, skip it, otherwise add it to `out`.
            for ext in intersection.extensions(&self) {
                if let Some(inter) = self.intersection(&ext) {
                    out.push(inter);
                }
            }
            out
        } else {
            vec![self.clone()]
        }
    }

    pub fn volume(&self) -> i64 {
        (self.x1 - self.x0 + 1) * (self.y1 - self.y0 + 1) * (self.z1 - self.z0 + 1)
    }

    pub fn intersection(&self, other: &Cuboid) -> Option<Cuboid> {
        let (left, right) = if self.x0 <= other.x0 {
            (self, other)
        } else {
            (other, self)
        };
        let x_seg = if left.x1 < right.x0 {
            return None;
        } else {
            (max(left.x0, right.x0), min(left.x1, right.x1))
        };

        let (left, right) = if self.y0 <= other.y0 {
            (self, other)
        } else {
            (other, self)
        };
        let y_seg = if left.y1 < right.y0 {
            return None;
        } else {
            (max(left.y0, right.y0), min(left.y1, right.y1))
        };

        let (left, right) = if self.z0 <= other.z0 {
            (self, other)
        } else {
            (other, self)
        };
        let z_seg = if left.z1 < right.z0 {
            return None;
        } else {
            (max(left.z0, right.z0), min(left.z1, right.z1))
        };

        Some(Cuboid::new(x_seg.0, x_seg.1, y_seg.0, y_seg.1, z_seg.0, z_seg.1).unwrap())
    }

    pub fn split(&self) -> AocResult<[Cuboid; 8]> {
        if self.x0 == self.x1 || self.y0 == self.y1 || self.z0 == self.z1 {
            return failure(format!("Cuboid {:?} is too small to split!", self));
        }
        let xlen = self.x1 - self.x0;
        let ylen = self.y1 - self.y0;
        let zlen = self.z1 - self.z0;

        // Segment lengths
        let xsl = [xlen / 2, xlen / 2 + 1];
        let ysl = [ylen / 2, ylen / 2 + 1];
        let zsl = [zlen / 2, zlen / 2 + 1];

        Ok([
            Cuboid::new(
                self.x0,
                self.x0 + xsl[0],
                self.y0,
                self.y0 + ysl[0],
                self.z0,
                self.z0 + zsl[0],
            )?,
            Cuboid::new(
                self.x0 + xsl[1],
                self.x1,
                self.y0,
                self.y0 + ysl[0],
                self.z0,
                self.z0 + zsl[0],
            )?,
            Cuboid::new(
                self.x0,
                self.x0 + xsl[0],
                self.y0 + ysl[1],
                self.y1,
                self.z0,
                self.z0 + zsl[0],
            )?,
            Cuboid::new(
                self.x0 + xsl[1],
                self.x1,
                self.y0 + ysl[1],
                self.y1,
                self.z0,
                self.z0 + zsl[0],
            )?,
            Cuboid::new(
                self.x0,
                self.x0 + xsl[0],
                self.y0,
                self.y0 + ysl[0],
                self.z0 + zsl[1],
                self.z1,
            )?,
            Cuboid::new(
                self.x0 + xsl[1],
                self.x1,
                self.y0,
                self.y0 + ysl[0],
                self.z0 + zsl[1],
                self.z1,
            )?,
            Cuboid::new(
                self.x0,
                self.x0 + xsl[0],
                self.y0 + ysl[1],
                self.y1,
                self.z0 + zsl[1],
                self.z1,
            )?,
            Cuboid::new(
                self.x0 + xsl[1],
                self.x1,
                self.y0 + ysl[1],
                self.y1,
                self.z0 + zsl[1],
                self.z1,
            )?,
        ])
    }
}

#[cfg(test)]
mod cuboid_tests {
    use super::*;

    #[test]
    fn cuboid_from_str() -> AocResult<()> {
        for s in ["x=-23..22,y=-17..33,z=-1..44"] {
            let c = Cuboid::from_str(s)?;
            assert_eq!(c, Cuboid::new(-23, 22, -17, 33, -1, 44)?);
        }
        Ok(())
    }

    #[test]
    fn cuboid_split() -> AocResult<()> {
        {
            let cs = Cuboid::new(0, 1, 0, 1, 0, 1)?.split()?;
            assert_eq!(
                cs,
                [
                    Cuboid::new(0, 0, 0, 0, 0, 0)?,
                    Cuboid::new(1, 1, 0, 0, 0, 0)?,
                    Cuboid::new(0, 0, 1, 1, 0, 0)?,
                    Cuboid::new(1, 1, 1, 1, 0, 0)?,
                    Cuboid::new(0, 0, 0, 0, 1, 1)?,
                    Cuboid::new(1, 1, 0, 0, 1, 1)?,
                    Cuboid::new(0, 0, 1, 1, 1, 1)?,
                    Cuboid::new(1, 1, 1, 1, 1, 1)?
                ]
            );
        }
        {
            let cs = Cuboid::new(-3, 3, -3, 3, -3, 3)?.split()?;
            assert_eq!(
                cs,
                [
                    Cuboid::new(-3, 0, -3, 0, -3, 0)?,
                    Cuboid::new(1, 3, -3, 0, -3, 0)?,
                    Cuboid::new(-3, 0, 1, 3, -3, 0)?,
                    Cuboid::new(1, 3, 1, 3, -3, 0)?,
                    Cuboid::new(-3, 0, -3, 0, 1, 3)?,
                    Cuboid::new(1, 3, -3, 0, 1, 3)?,
                    Cuboid::new(-3, 0, 1, 3, 1, 3)?,
                    Cuboid::new(1, 3, 1, 3, 1, 3)?,
                ]
            );
        }
        Ok(())
    }

    #[test]
    fn cuboid_intersection() -> AocResult<()> {
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            let c2 = c1.clone();
            assert_eq!(c1.intersection(&c2).unwrap(), c1);
        }
        {
            let c1 = Cuboid::new(-1, 1, -1, 1, -1, 1)?;
            let c2 = Cuboid::new(0, 0, 0, 0, 0, 0)?;
            assert_eq!(c1.intersection(&c2).unwrap(), c2);
            assert_eq!(c2.intersection(&c1).unwrap(), c2);
        }
        {
            let c1 = Cuboid::new(-1, 1, -1, 1, -1, 1)?;
            let c2 = Cuboid::new(0, 2, 0, 2, 0, 2)?;
            assert_eq!(
                c1.intersection(&c2).unwrap(),
                Cuboid::new(0, 1, 0, 1, 0, 1)?
            );
            assert_eq!(
                c2.intersection(&c1).unwrap(),
                Cuboid::new(0, 1, 0, 1, 0, 1)?
            );
        }
        {
            let c1 = Cuboid::new(-1, 1, -1, 1, -1, 1)?;
            let c2 = Cuboid::new(-2, 2, 2, 2, 2, 2)?;
            assert_eq!(c1.intersection(&c2), None);
            assert_eq!(c2.intersection(&c1), None);
        }
        {
            let c1 = Cuboid::new(0, 1, 3, 4, -5, -3)?;
            let c2 = Cuboid::new(-2, 2, -9, 6, -4, -4)?;
            assert_eq!(
                c1.intersection(&c2).unwrap(),
                Cuboid::new(0, 1, 3, 4, -4, -4)?
            );
            assert_eq!(
                c2.intersection(&c1).unwrap(),
                Cuboid::new(0, 1, 3, 4, -4, -4)?
            );
        }
        Ok(())
    }
    #[test]
    fn cuboid_difference() -> AocResult<()> {
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            assert_eq!(c1.difference(&c1).len(), 0);
        }
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            let c2 = Cuboid::new(2, 3, 2, 3, 2, 3)?;
            assert_eq!(c1.difference(&c2)[0], c1);
        }
        {
            let c1 = Cuboid::new(0, 2, 0, 2, 0, 2)?;
            let c2 = Cuboid::new(1, 1, 1, 1, 1, 1)?;
            let mut d = c1.difference(&c2);
            d.as_mut_slice().sort();
            let mut d2 = Vec::new();
            for x in 0..=2 {
                for y in 0..=2 {
                    for z in 0..=2 {
                        if (x, y, z) == (1, 1, 1) {
                            continue;
                        }
                        d2.push(Cuboid::new(x, x, y, y, z, z)?);
                    }
                }
            }
            d2.as_mut_slice().sort();
            assert_eq!(d, d2);
        }
        Ok(())
    }
}

/// Contains disjoint cuboids
#[derive(Debug)]
pub struct PolyCuboid {
    cuboids: Vec<Cuboid>,
}

impl fmt::Display for PolyCuboid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.iter() {
            write!(f, "{}\n", c)?;
        }
        Ok(())
    }
}

impl PolyCuboid {
    pub fn new() -> Self {
        Self {
            cuboids: Vec::new(),
        }
    }

    pub fn volume(&self) -> i64 {
        self.iter().fold(0, |acc, c| acc + c.volume())
    }

    pub fn iter(&self) -> Iter<'_, Cuboid> {
        self.cuboids.iter()
    }

    pub fn insert(&mut self, other: &Cuboid) {
        let mut others = vec![other.clone()];
        let mut overlap = true;
        let mut skip_i = 0;
        while overlap {
            overlap = false;
            for (i, c) in self.iter().skip(skip_i).enumerate() {
                for (j, other) in others.iter().enumerate() {
                    if c.contains(other) {
                        others.swap_remove(j);
                        overlap = true;
                        break;
                    }
                    if other.intersection(c).is_some() {
                        let mut diff = other.difference(c);
                        others.swap_remove(j);
                        others.append(&mut diff);
                        overlap = true;
                        break;
                    }
                }
                if !overlap {
                    skip_i = i;
                }
            }
        }
        self.cuboids.append(&mut others);
    }

    pub fn delete(&mut self, other: &Cuboid) {
        let mut post_delete: Vec<Cuboid> = Vec::new();
        for c in self.iter() {
            let mut diff = c.difference(other);
            post_delete.append(&mut diff);
        }
        self.cuboids = post_delete;
    }
}

#[cfg(test)]
mod polycuboid_tests {
    use super::*;

    #[test]
    fn polycuboid_insert() -> AocResult<()> {
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            let mut p = PolyCuboid::new();
            p.insert(&c1);
            assert_eq!(p.cuboids[0], c1);
            assert_eq!(p.cuboids.len(), 1);
            p.insert(&c1);
            assert_eq!(p.cuboids[0], c1);
            assert_eq!(p.cuboids.len(), 1);
            assert_eq!(p.volume(), 8);
        }
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            let c2 = Cuboid::new(1, 2, 1, 2, 1, 2)?;
            let mut p = PolyCuboid::new();
            p.insert(&c1);
            p.insert(&c2);
            assert_eq!(p.volume(), 15);
        }
        Ok(())
    }
    #[test]
    fn polycuboid_delete() -> AocResult<()> {
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            let mut p = PolyCuboid::new();
            p.delete(&c1);
            assert_eq!(p.volume(), 0);
            p.insert(&c1);
            assert_eq!(p.volume(), 8);
            p.delete(&c1);
            assert_eq!(p.volume(), 0);
        }
        {
            let c1 = Cuboid::new(0, 1, 0, 1, 0, 1)?;
            let c2 = Cuboid::new(1, 2, 1, 2, 1, 2)?;
            let mut p = PolyCuboid::new();
            p.insert(&c1);
            assert_eq!(p.volume(), 8);
            p.insert(&c2);
            assert_eq!(p.volume(), 15);
            p.delete(&c1);
            assert_eq!(p.volume(), 7);
            p.delete(&c2);
            assert_eq!(p.volume(), 0);
        }
        {
            let c1 = Cuboid::new(0, 1, -1, 1, 3, 5)?;
            let c2 = Cuboid::new(-1, 2, -1, 0, 4, 9)?;
            let c3 = Cuboid::new(3, 5, -1, 4, 1, 2)?;
            let c4 = Cuboid::new(0, 0, 0, 0, 0, 0)?;
            let c5 = Cuboid::new(-9, 5, -9, 5, -9, 5)?;
            let mut p = PolyCuboid::new();
            let mut ph = PolyCuboid::new();
            p.insert(&c1);
            ph.insert(&c1);
            assert_eq!(p.volume(), ph.volume());
            p.insert(&c2);
            ph.insert(&c2);
            assert_eq!(p.volume(), ph.volume());
            p.insert(&c3);
            ph.insert(&c3);
            assert_eq!(p.volume(), ph.volume());
            p.delete(&c2);
            ph.delete(&c2);
            assert_eq!(p.volume(), ph.volume());
            p.delete(&c1);
            ph.delete(&c1);
            assert_eq!(p.volume(), ph.volume());
            p.insert(&c4);
            ph.insert(&c4);
            assert_eq!(p.volume(), ph.volume());
            p.delete(&c3);
            ph.delete(&c3);
            assert_eq!(p.volume(), ph.volume());
            p.insert(&c5);
            ph.insert(&c5);
            assert_eq!(p.volume(), ph.volume());
            p.delete(&c4);
            ph.delete(&c4);
            assert_eq!(p.volume(), ph.volume());
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PolyHashCuboid {
    voxels: HashSet<(i64, i64, i64)>,
}

impl PolyHashCuboid {
    pub fn new() -> Self {
        Self {
            voxels: HashSet::new(),
        }
    }

    pub fn volume(&self) -> i64 {
        self.voxels.len().try_into().unwrap()
    }

    pub fn insert(&mut self, other: &Cuboid) {
        for x in other.x0..=other.x1 {
            for y in other.y0..=other.y1 {
                for z in other.z0..=other.z1 {
                    self.voxels.insert((x, y, z));
                }
            }
        }
    }

    pub fn delete(&mut self, other: &Cuboid) {
        for x in other.x0..=other.x1 {
            for y in other.y0..=other.y1 {
                for z in other.z0..=other.z1 {
                    self.voxels.remove(&(x, y, z));
                }
            }
        }
    }
}
