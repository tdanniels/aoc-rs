use aoc_util::{get_cli_arg, AocResult};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;

const N_ALIGN: u32 = 12;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct Point3 {
    x: i64,
    y: i64,
    z: i64,
}

impl Add for Point3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Point3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Neg for Point3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// Inner product.
impl Mul for Point3 {
    type Output = i64;
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl FromStr for Point3 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coords: Vec<&str> = s.split(',').collect();

        let x_fromstr = coords[0].parse::<i64>()?;
        let y_fromstr = coords[1].parse::<i64>()?;
        let z_fromstr = coords[2].parse::<i64>()?;

        Ok(Point3 {
            x: x_fromstr,
            y: y_fromstr,
            z: z_fromstr,
        })
    }
}

impl Point3 {
    fn new(x: i64, y: i64, z: i64) -> Self {
        Point3 { x, y, z }
    }

    fn magnitude(&self) -> i64 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }

    fn orient(&self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::PlusX => Point3::new(self.x, self.y, self.z),
            Orientation::PlusY => Point3::new(-self.y, self.x, self.z),
            Orientation::PlusZ => Point3::new(self.z, self.y, -self.x),
            Orientation::MinusX => Point3::new(-self.x, self.y, -self.z),
            Orientation::MinusY => Point3::new(self.y, -self.x, self.z),
            Orientation::MinusZ => Point3::new(-self.z, self.y, self.x),
        }
    }

    fn rotate(&self, orientation: Orientation, rotation: Rotation) -> Self {
        match orientation {
            Orientation::PlusX | Orientation::MinusX => match rotation {
                Rotation::_0 => Point3::new(self.x, self.y, self.z),
                Rotation::_90 => Point3::new(self.x, -self.z, self.y),
                Rotation::_180 => Point3::new(self.x, -self.y, -self.z),
                Rotation::_270 => Point3::new(self.x, self.z, -self.y),
            },
            Orientation::PlusY | Orientation::MinusY => match rotation {
                Rotation::_0 => Point3::new(self.x, self.y, self.z),
                Rotation::_90 => Point3::new(self.z, self.y, -self.x),
                Rotation::_180 => Point3::new(-self.x, self.y, -self.z),
                Rotation::_270 => Point3::new(-self.z, self.y, self.x),
            },
            Orientation::PlusZ | Orientation::MinusZ => match rotation {
                Rotation::_0 => Point3::new(self.x, self.y, self.z),
                Rotation::_90 => Point3::new(-self.y, self.x, self.z),
                Rotation::_180 => Point3::new(-self.x, -self.y, self.z),
                Rotation::_270 => Point3::new(self.y, -self.x, self.z),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Orientation {
    PlusX,
    PlusY,
    PlusZ,
    MinusX,
    MinusY,
    MinusZ,
}

#[derive(Clone, Copy, Debug)]
enum Rotation {
    _0,
    _90,
    _180,
    _270,
}

#[derive(Clone, Debug)]
struct Problem {
    scanners: Vec<Scanner>,
}

const ORIENTATIONS: [Orientation; 6] = [
    Orientation::PlusX,
    Orientation::PlusY,
    Orientation::PlusZ,
    Orientation::MinusX,
    Orientation::MinusY,
    Orientation::MinusZ,
];

const ROTATIONS: [Rotation; 4] =
    [Rotation::_0, Rotation::_90, Rotation::_180, Rotation::_270];

#[derive(Clone, Copy, Debug)]
struct CoordinateSystem {
    orientation: Orientation,
    rotation: Rotation,
}

#[derive(Clone, Debug)]
struct Scanner {
    data: Vec<Point3>,
    /// None indicates an position.
    position: Option<Point3>,
    /// None indicates an unknown coordinate system.
    coordinate_system: Option<CoordinateSystem>,
    /// sorted_squared_dists[i] = {d, j, k} is the squared distance
    /// from data point j to data point k. Note that if {d, j, k}
    /// is present, {d, k, j} won't be due to deduplication.
    /// The case where j == k is also deduplicated.
    sorted_squared_dists: Vec<(i64, usize, usize)>,
}

impl Scanner {
    fn new(
        data: Vec<Point3>,
        position: Option<Point3>,
        coordinate_system: Option<CoordinateSystem>,
    ) -> Self {
        let mut squared_dists = BinaryHeap::new();
        for (i, p0) in data.iter().enumerate() {
            squared_dists.append(
                &mut data
                    .iter()
                    .enumerate()
                    .skip(i + 1) // Avoid d_i * d_i and counting distances twice.
                    .map(|(j, p1)| ((*p1 - *p0) * (*p1 - *p0), i, j))
                    .collect::<BinaryHeap<_>>(),
            );
        }
        Scanner {
            data,
            position,
            coordinate_system,
            sorted_squared_dists: squared_dists.into_sorted_vec(),
        }
    }

    /// Try to derive the coordinate system and offset of `other` relative to `self`.
    fn try_derive_coordinate_system_and_offset(
        &self,
        other: &Scanner,
    ) -> Option<(CoordinateSystem, Point3)> {
        let mut sqdist_to_idx_pairs = HashMap::new();
        for sqd in &self.sorted_squared_dists {
            let mut start = 0;
            while let Ok(idx) =
                &other.sorted_squared_dists[start..].binary_search_by_key(&sqd.0, |&d| d.0)
            {
                let entry = sqdist_to_idx_pairs.entry(sqd).or_insert(Vec::new());
                entry.push((
                    (sqd.1, sqd.2),
                    (
                        other.sorted_squared_dists[start + *idx].1,
                        other.sorted_squared_dists[start + *idx].2,
                    ),
                ));

                if start + *idx == other.sorted_squared_dists.len() - 1 {
                    break;
                } else {
                    start += *idx + 1;
                }
            }
        }
        // Find the indices of self.data which occur at least NUM_ALIGN - 1 times (in either
        // position) in sqdist_to_idx_pairs .
        let mut self_index_counts = HashMap::new();
        let mut other_index_counts = HashMap::new();
        for (_, v) in sqdist_to_idx_pairs {
            for e in v {
                let entry = self_index_counts.entry(e.0 .0).or_insert(0);
                *entry += 1;
                let entry = self_index_counts.entry(e.0 .1).or_insert(0);
                *entry += 1;
                let entry = other_index_counts.entry(e.1 .0).or_insert(0);
                *entry += 1;
                let entry = other_index_counts.entry(e.1 .1).or_insert(0);
                *entry += 1;
            }
        }

        let self_indices = self_index_counts
            .into_iter()
            .filter(|&(_, v)| v >= N_ALIGN - 1)
            .map(|(k, _)| k)
            .collect::<Vec<_>>();
        let other_indices = other_index_counts
            .into_iter()
            .filter(|&(_, v)| v >= N_ALIGN - 1)
            .map(|(k, _)| k)
            .collect::<Vec<_>>();

        if self_indices.len() < N_ALIGN as usize || other_indices.len() < N_ALIGN as usize {
            return None;
        }

        // Find the alignment.
        let aligned_self_points = {
            let mut asp = Vec::with_capacity(N_ALIGN as usize);
            for i in &self_indices {
                asp.push(self.data[*i]);
            }
            asp
        };
        let mut ori = None;
        let mut rot = None;
        let mut ofs = None;
        for orientation in ORIENTATIONS {
            for rotation in ROTATIONS {
                let mut aligned_other_points = Vec::with_capacity(N_ALIGN as usize);
                for i in &other_indices {
                    aligned_other_points.push(other.data[*i]);
                }
                aligned_other_points = aligned_other_points
                    .iter()
                    .map(|p| p.orient(orientation))
                    .map(|p| p.rotate(orientation, rotation))
                    .collect();
                let mut offsets2counts = HashMap::new();
                for sp in &aligned_self_points {
                    for op in &aligned_other_points {
                        let entry = offsets2counts.entry(*sp - *op).or_insert(0);
                        *entry += 1;
                    }
                }
                if let Some((true_ofs, _)) =
                    offsets2counts.iter().find(|(_, v)| **v >= N_ALIGN)
                {
                    ofs = Some(true_ofs.clone());
                    ori = Some(orientation);
                    rot = Some(rotation);
                    break;
                }
            }
        }
        if ori.is_none() || rot.is_none() {
            return None;
        }

        Some((
            CoordinateSystem {
                orientation: ori.unwrap(),
                rotation: rot.unwrap(),
            },
            ofs.unwrap(),
        ))
    }

    fn align_measurements(&mut self, coordinate_system: CoordinateSystem, offset: Point3) {
        self.data = self
            .data
            .iter()
            .map(|p| p.orient(coordinate_system.orientation))
            .map(|p| p.rotate(coordinate_system.orientation, coordinate_system.rotation))
            .map(|p| p + offset)
            .collect();
    }
}

fn parse_input(lines: &Vec<String>) -> AocResult<Problem> {
    let mut scanners = Vec::new();
    let mut data = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if l.starts_with("---") {
            data.clear();
        } else if l.trim().is_empty() {
            scanners.push(Scanner::new(data.clone(), None, None));
            continue;
        } else {
            let p = Point3::from_str(l)?;
            data.push(p);
            if i == lines.len() - 1 {
                scanners.push(Scanner::new(data.clone(), None, None));
            }
        }
    }
    Ok(Problem { scanners })
}

fn solve(mut problem: Problem) -> AocResult<(usize, i64)> {
    problem.scanners[0].coordinate_system = Some(CoordinateSystem {
        orientation: ORIENTATIONS[0],
        rotation: ROTATIONS[0],
    });
    problem.scanners[0].position = Some(Point3 { x: 0, y: 0, z: 0 });
    let mut scanners_to_align: Vec<usize> = (1..problem.scanners.len()).collect();
    let mut aligned_scanners: Vec<usize> = vec![0];

    // It's wasteful to try to force the 'chaining' of scanners from scanner 0,
    // since we waste work on aligning scanners that, while they may align, aren't
    // the next pair in the chain. Is *is* simpler this way though.
    while scanners_to_align.len() > 0 {
        let mut did_align = false;
        'outer: for aligned_scanner_idx in &aligned_scanners {
            for (i, scanner_idx) in scanners_to_align.iter().enumerate() {
                if let Some((cs, position)) = problem.scanners[*aligned_scanner_idx]
                    .try_derive_coordinate_system_and_offset(&problem.scanners[*scanner_idx])
                {
                    problem.scanners[*scanner_idx].coordinate_system = Some(cs);
                    problem.scanners[*scanner_idx].position = Some(position);
                    problem.scanners[*scanner_idx].align_measurements(cs, position);
                    did_align = true;
                    aligned_scanners.push(*scanner_idx);
                    scanners_to_align.swap_remove(i);
                    break 'outer;
                }
            }
        }
        if !did_align {
            panic!("Couldn't align any scanners");
        }
    }

    let mut dists = BinaryHeap::new();
    for s1 in &problem.scanners {
        for s2 in &problem.scanners {
            dists.push((s1.position.unwrap() - s2.position.unwrap()).magnitude());
        }
    }

    let beacons: HashSet<Point3> = problem
        .scanners
        .into_iter()
        .flat_map(|s| s.data.into_iter())
        .collect();

    Ok((beacons.len(), *dists.peek().unwrap()))
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    println!("Part 1: {}", solve(parse_input(&lines)?)?.0);
    println!("Part 2: {}", solve(parse_input(&lines)?)?.1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn point_align() -> AocResult<()> {
        let p = Point3::new(1, 2, 3);
        assert_eq!(
            p.orient(Orientation::PlusX)
                .rotate(Orientation::PlusX, Rotation::_90)
                .rotate(Orientation::PlusX, Rotation::_90)
                .rotate(Orientation::PlusX, Rotation::_90)
                .rotate(Orientation::PlusX, Rotation::_90),
            p
        );
        assert_eq!(
            p.orient(Orientation::PlusX)
                .rotate(Orientation::PlusX, Rotation::_180)
                .rotate(Orientation::PlusX, Rotation::_90)
                .rotate(Orientation::PlusX, Rotation::_270),
            p.rotate(Orientation::PlusX, Rotation::_180)
        );
        Ok(())
    }

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(solve(parse_input(&lines)?)?.0, 79);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(solve(parse_input(&lines)?)?.0, 308);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(solve(parse_input(&lines)?)?.1, 3621);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(solve(parse_input(&lines)?)?.1, 12124);
        Ok(())
    }
}
