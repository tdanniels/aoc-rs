use aoc_util::{failure, get_cli_arg, AocResult, Cuboid, PolyCuboid};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Debug)]
struct Op {
    to_state: bool,
    cuboid: Cuboid,
}

fn parse_input(lines: &[String]) -> AocResult<Vec<Op>> {
    lines
        .iter()
        .map(|l| {
            let mut split = l.split_whitespace();
            let to_state = match split.next() {
                Some("on") => true,
                Some("off") => false,
                _ => failure("Bad on/off")?,
            };
            let cuboid = split.next().ok_or("No cuboid?")?.parse::<Cuboid>()?;
            Ok(Op { to_state, cuboid })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn part_1(ops: &[Op]) -> AocResult<i64> {
    let filter_cuboid = Cuboid::new(-50, 50, -50, 50, -50, 50)?;
    let filtered_ops: Vec<&Op> = ops
        .iter()
        .filter(|o| filter_cuboid.contains(&o.cuboid))
        .collect();
    let mut polycuboid = PolyCuboid::new();
    for op in filtered_ops {
        if op.to_state {
            polycuboid.insert(&op.cuboid);
        } else {
            polycuboid.delete(&op.cuboid);
        }
    }

    Ok(polycuboid.volume())
}

fn part_2(ops: &Vec<Op>) -> AocResult<i64> {
    let mut polycuboid = PolyCuboid::new();
    for op in ops {
        if op.to_state {
            polycuboid.insert(&op.cuboid);
        } else {
            polycuboid.delete(&op.cuboid);
        }
    }
    Ok(polycuboid.volume())
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    let ops = parse_input(&lines)?;
    println!("Part 1: {}", part_1(&ops)?);
    println!("Part 2: {}", part_2(&ops)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn simple_test1() -> AocResult<()> {
        let ops = vec![Op {
            to_state: true,
            cuboid: Cuboid::new(0, 1, 0, 1, 0, 1)?,
        }];
        assert_eq!(part_1(&ops)?, 8);
        Ok(())
    }

    #[test]
    fn simple_test2() -> AocResult<()> {
        let ops = vec![
            Op {
                to_state: true,
                cuboid: Cuboid::new(0, 1, 0, 1, 0, 1)?,
            },
            Op {
                to_state: false,
                cuboid: Cuboid::new(0, 1, 0, 1, 0, 1)?,
            },
        ];
        assert_eq!(part_1(&ops)?, 0);
        Ok(())
    }

    #[test]
    fn simple_test3() -> AocResult<()> {
        let ops = vec![
            Op {
                to_state: true,
                cuboid: Cuboid::new(0, 1, 0, 1, 0, 1)?,
            },
            Op {
                to_state: true,
                cuboid: Cuboid::new(2, 3, 2, 3, 2, 3)?,
            },
        ];
        assert_eq!(part_1(&ops)?, 16);
        Ok(())
    }

    #[test]
    fn simple_test4() -> AocResult<()> {
        let ops = vec![
            Op {
                to_state: true,
                cuboid: Cuboid::new(0, 1, 0, 1, 0, 1)?,
            },
            Op {
                to_state: false,
                cuboid: Cuboid::new(2, 3, 2, 3, 2, 3)?,
            },
        ];
        assert_eq!(part_1(&ops)?, 8);
        Ok(())
    }

    #[test]
    fn simple_test5() -> AocResult<()> {
        let vs = vec![
            "on x=10..12,y=10..12,z=10..12".to_string(),
            "on x=11..13,y=11..13,z=11..13".to_string(),
            "off x=9..11,y=9..11,z=9..11".to_string(),
            "on x=10..10,y=10..10,z=10..10".to_string(),
        ];
        let ops = parse_input(&vs)?;
        assert_eq!(part_1(&ops)?, 39);
        Ok(())
    }

    #[test]
    fn simple_test6() -> AocResult<()> {
        let vs = vec![
            "on x=-20..26,y=-36..17,z=-47..7".to_string(),
            "on x=-20..33,y=-21..23,z=-26..28".to_string(),
            "on x=-22..28,y=-29..23,z=-38..16".to_string(),
            "on x=-46..7,y=-6..46,z=-50..-1".to_string(),
            "on x=-49..1,y=-3..46,z=-24..28".to_string(),
            "on x=2..47,y=-22..22,z=-23..27".to_string(),
            "on x=-27..23,y=-28..26,z=-21..29".to_string(),
            "on x=-39..5,y=-6..47,z=-3..44".to_string(),
            "on x=-30..21,y=-8..43,z=-13..34".to_string(),
            "on x=-22..26,y=-27..20,z=-29..19".to_string(),
            "off x=-48..-32,y=26..41,z=-47..-37".to_string(),
            "on x=-12..35,y=6..50,z=-50..-2".to_string(),
            "off x=-48..-32,y=-32..-16,z=-15..-5".to_string(),
            "on x=-18..26,y=-33..15,z=-7..46".to_string(),
            "off x=-40..-22,y=-38..-28,z=23..41".to_string(),
            "on x=-16..35,y=-41..10,z=-47..6".to_string(),
            "off x=-32..-23,y=11..30,z=-14..3".to_string(),
            "on x=-49..-5,y=-3..45,z=-29..18".to_string(),
        ];
        let ops = parse_input(&vs)?;
        assert_eq!(part_1(&ops)?, 592902);
        Ok(())
    }

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let ops = parse_input(&lines)?;
        assert_eq!(part_1(&ops)?, 590784);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let ops = parse_input(&lines)?;
        assert_eq!(part_1(&ops)?, 561032);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let ops = parse_input(&lines)?;
        assert_eq!(part_2(&ops)?, 39769202357779);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let ops = parse_input(&lines)?;
        assert_eq!(part_2(&ops)?, 1322825263376414);
        Ok(())
    }
}
