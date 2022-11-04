use crate::errors::{failure, AocResult};

use std::cmp::{max, min};
use std::collections::HashSet;
use std::error;
use std::fmt;
use std::num::ParseIntError;
use std::slice::Iter;
use std::str::FromStr;

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
                start + s[start..].find(',').ok_or("No suffix \",\"?")?
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
        } else if other.contains(self) {
            vec![other.clone()]
        } else if self.intersects(other) {
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
        debug_assert!(out.iter().all(|c| !c.intersects(self)));
        debug_assert!(out.iter().enumerate().all(|(i, c1)| out
            .iter()
            .enumerate()
            .all(|(j, c2)| i == j || !c1.intersects(c2))));
        out
    }

    pub fn difference(&self, other: &Cuboid) -> Vec<Cuboid> {
        if other.contains(self) {
            vec![]
        } else if let Some(intersection) = self.intersection(other) {
            let mut out = Vec::with_capacity(26);
            // Extend `intersection` in all 26 possible directions, and take the
            // intersection of `ext` and `self` to obtain a possible partial difference
            // cuboid. If the new intersection is empty, skip it, otherwise add it to `out`.
            for ext in intersection.extensions(self) {
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

    pub fn intersects(&self, other: &Cuboid) -> bool {
        let (left, right) = if self.x0 <= other.x0 {
            (self, other)
        } else {
            (other, self)
        };
        if left.x1 < right.x0 {
            return false;
        }

        let (left, right) = if self.y0 <= other.y0 {
            (self, other)
        } else {
            (other, self)
        };
        if left.y1 < right.y0 {
            return false;
        }

        let (left, right) = if self.z0 <= other.z0 {
            (self, other)
        } else {
            (other, self)
        };
        if left.z1 < right.z0 {
            return false;
        }

        true
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
        {
            let s = "x=-23..22,y=-17..33,z=-1..44";
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
#[derive(Default, Debug)]
pub struct PolyCuboid {
    cuboids: Vec<Cuboid>,
}

impl fmt::Display for PolyCuboid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.iter() {
            writeln!(f, "{}", c)?;
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
                    if other.intersects(c) {
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

#[derive(Default, Debug)]
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
