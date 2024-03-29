use aoc_util::{errors::AocResult, graph::UnweightedUndirectedGraph, io::get_cli_arg};
use std::collections::HashSet;

/// It appears to be an unstated fact of this problem that large caves
/// are never directly connected to other large caves, otherwise there would
/// be an infinite number of paths.
fn part_1(graph: &UnweightedUndirectedGraph) -> AocResult<u64> {
    let visited_small_caves: HashSet<&str> = HashSet::new();
    count_paths_to_end(graph, "start", 0, &visited_small_caves, false, None)
}

fn part_2(graph: &UnweightedUndirectedGraph) -> AocResult<u64> {
    let visited_small_caves: HashSet<&str> = HashSet::new();
    count_paths_to_end(graph, "start", 0, &visited_small_caves, true, None)
}

fn count_paths_to_end(
    graph: &UnweightedUndirectedGraph,
    node: &str,
    prev_count: u64,
    visited_small_caves: &HashSet<&str>,
    allow_twice: bool,
    twice_node: Option<&str>,
) -> AocResult<u64> {
    if node == "end" {
        return Ok(1);
    }

    let mut count = 0;

    let mut visited_small_caves = visited_small_caves.clone();
    if node.chars().all(char::is_lowercase) {
        visited_small_caves.insert(node);
    }

    let mut new_twice_node = twice_node;
    for neighbour in graph.neighbour_names(node)? {
        if visited_small_caves.get(neighbour).is_some() {
            if allow_twice && twice_node.is_none() && neighbour != "start" {
                new_twice_node = Some(neighbour);
            } else {
                continue;
            }
        }

        count += count_paths_to_end(
            graph,
            neighbour,
            prev_count,
            &visited_small_caves,
            allow_twice,
            new_twice_node,
        )?;
        new_twice_node = twice_node;
    }
    Ok(prev_count + count)
}

fn main() -> AocResult<()> {
    let graph = UnweightedUndirectedGraph::from_file(&get_cli_arg()?)?;
    println!("Part 1: {}", part_1(&graph)?);
    println!("Part 2: {}", part_2(&graph)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::io::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let graph = UnweightedUndirectedGraph::from_file(&get_test_file(file!())?)?;
        assert_eq!(part_1(&graph)?, 226);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let graph = UnweightedUndirectedGraph::from_file(&get_input_file(file!())?)?;
        assert_eq!(part_1(&graph)?, 3679);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let graph = UnweightedUndirectedGraph::from_file(&get_test_file(file!())?)?;
        assert_eq!(part_2(&graph)?, 3509);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let graph = UnweightedUndirectedGraph::from_file(&get_input_file(file!())?)?;
        assert_eq!(part_2(&graph)?, 107395);
        Ok(())
    }
}
