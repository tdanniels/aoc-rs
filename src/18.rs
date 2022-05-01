use aoc_util::{failure, get_cli_arg, AocResult, Node, NodeWrapper};
use std::cmp;
use std::fs::File;
use std::io::{self, BufRead};

fn add(left: &NodeWrapper, right: &NodeWrapper) -> AocResult<NodeWrapper> {
    let sum = NodeWrapper::from(Node::new(None));
    sum.set_left(Some(left));
    sum.set_right(Some(right));
    reduce(&sum)?;
    Ok(sum)
}

fn reduce(node: &NodeWrapper) -> AocResult<()> {
    loop {
        if try_explode(node)? {
            continue;
        }
        if try_split(node) {
            continue;
        }
        break;
    }
    Ok(())
}

fn try_explode(node: &NodeWrapper) -> AocResult<bool> {
    let nodes_dfs_order = node.depth_first_iter().collect::<Vec<_>>();
    if let Some((exploding_node, _)) = nodes_dfs_order.iter().find(|(_, depth)| *depth == 5) {
        assert!(exploding_node.is_leaf() && exploding_node.has_data());
        let exploding_node = exploding_node.get_parent().unwrap();
        assert!(!exploding_node.is_leaf() && !exploding_node.has_data());
        let left_ex = exploding_node.get_left().unwrap();
        let left_ex_val = left_ex.get_data().unwrap();
        let left_ex_idx = nodes_dfs_order
            .iter()
            .enumerate()
            .find(|(_, (node, _))| node.inner().as_ptr() == left_ex.inner().as_ptr())
            .unwrap()
            .0;
        let right_ex = exploding_node.get_right().unwrap();
        let right_ex_val = exploding_node.get_right().unwrap().get_data().unwrap();
        let right_ex_idx = nodes_dfs_order
            .iter()
            .enumerate()
            .find(|(_, (node, _))| node.inner().as_ptr() == right_ex.inner().as_ptr())
            .unwrap()
            .0;

        if let Some(left_collider) = nodes_dfs_order[..left_ex_idx]
            .iter()
            .rev()
            .find(|(node, _)| node.has_data())
        {
            left_collider
                .0
                .set_data(Some(left_ex_val + left_collider.0.get_data().unwrap()));
        }

        if right_ex_idx < nodes_dfs_order.len() - 1 {
            if let Some(right_collider) = nodes_dfs_order[right_ex_idx + 1..]
                .iter()
                .find(|(node, _)| node.has_data())
            {
                right_collider
                    .0
                    .set_data(Some(right_ex_val + right_collider.0.get_data().unwrap()));
            }
        }

        exploding_node.set_left(None);
        exploding_node.set_right(None);
        exploding_node.set_data(Some(0));
        return Ok(true);
    }
    Ok(false)
}

fn try_split(node: &NodeWrapper) -> bool {
    if let Some((large_node, _)) = node.depth_first_iter().find(|(node, _)| {
        if let Some(data) = node.get_data() {
            data >= 10
        } else {
            false
        }
    }) {
        let large_node = NodeWrapper::from(large_node);
        let data = large_node.get_data().unwrap();
        let new_left = Node::new(Some(data / 2));
        let new_right = Node::new(Some(data / 2 + if data % 2 != 0 { 1 } else { 0 }));
        large_node.set_left(Some(&new_left.into()));
        large_node.set_right(Some(&new_right.into()));
        large_node.set_data(None);
        return true;
    }
    false
}

fn magnitude(node: &NodeWrapper) -> i64 {
    if node.is_leaf() {
        unreachable!("Shouldn't happen");
    }

    let left_mag = if let Some(left_data) = node.get_left().unwrap().get_data() {
        left_data
    } else {
        magnitude(&node.get_left().unwrap())
    };

    let right_mag = if let Some(right_data) = node.get_right().unwrap().get_data() {
        right_data
    } else {
        magnitude(&node.get_right().unwrap())
    };

    3 * left_mag + 2 * right_mag
}

fn parse_input(lines: &Vec<String>) -> AocResult<Vec<Vec<NodeWrapper>>> {
    let mut problems = Vec::new();
    let mut problem = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if !l.is_ascii() {
            return failure("Non-ascii line");
        }
        if l.trim() == "" {
            if problem.len() > 0 {
                problems.push(problem);
                problem = Vec::new();
            }
            continue;
        }
        problem.push(NodeWrapper::from_ascii(l.as_bytes())?);

        if i == lines.len() - 1 {
            problems.push(problem);
            problem = Vec::new();
        }
    }
    Ok(problems)
}

fn part_1(mut problem: Vec<NodeWrapper>) -> AocResult<i64> {
    let mut sum = problem.remove(0);
    for num in problem.into_iter() {
        sum = add(&sum, &num)?;
    }
    Ok(magnitude(&sum))
}

fn part_2(problem: Vec<NodeWrapper>) -> AocResult<i64> {
    let mut max = 0;
    for (i, num_a) in problem.iter().enumerate() {
        for (j, num_b) in problem.iter().enumerate() {
            if i == j {
                continue;
            }

            // Super inefficient, but good enough for now.
            let num_a_clone = NodeWrapper::from_ascii(num_a.to_string().as_bytes())?;
            let num_b_clone = NodeWrapper::from_ascii(num_b.to_string().as_bytes())?;
            max = cmp::max(
                max,
                magnitude(&add(&num_a_clone, &num_b_clone)?),
            );

            let num_a_clone = NodeWrapper::from_ascii(num_a.to_string().as_bytes())?;
            let num_b_clone = NodeWrapper::from_ascii(num_b.to_string().as_bytes())?;
            max = cmp::max(
                max,
                magnitude(&add(&num_b_clone, &num_a_clone)?),
            );
        }
    }
    Ok(max)
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    println!("Part 1: {}", part_1(parse_input(&lines)?.remove(0))?);
    println!("Part 2: {}", part_2(parse_input(&lines)?.remove(0))?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test_1() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(0))?, 3488);
        Ok(())
    }

    #[test]
    fn part_1_test_2() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(1))?, 143);
        Ok(())
    }

    #[test]
    fn part_1_test_3() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(2))?, 1384);
        Ok(())
    }

    #[test]
    fn part_1_test_4() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(3))?, 445);
        Ok(())
    }

    #[test]
    fn part_1_test_5() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(4))?, 791);
        Ok(())
    }

    #[test]
    fn part_1_test_6() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(5))?, 1137);
        Ok(())
    }

    #[test]
    fn part_1_test_7() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(6))?, 4140);
        Ok(())
    }

    #[test]
    fn part_1_test_8() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(7))?, 1384);
        Ok(())
    }

    #[test]
    fn part_1_test_9() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(8))?, 1384);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(parse_input(&lines)?.remove(0))?, 3411);
        Ok(())
    }

    #[test]
    fn part_2_test_1() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_2(parse_input(&lines)?.remove(6))?, 3993);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_2(parse_input(&lines)?.remove(0))?, 4680);
        Ok(())
    }
}
