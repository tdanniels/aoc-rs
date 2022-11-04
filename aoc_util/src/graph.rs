use crate::errors::{failure, AocResult};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};

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

            for e in edge.iter().take(2) {
                if node2index.get(e).is_none() {
                    nodes.push(e.clone());
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
