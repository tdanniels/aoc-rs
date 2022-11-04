use crate::errors::{failure, AocResult};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};

/// A graph in adjacency list form.
#[derive(Debug)]
pub struct UnweightedUndirectedGraph {
    edges: Vec<Vec<usize>>,
    names: Vec<String>,
    name2node: HashMap<String, usize>,
}

impl UnweightedUndirectedGraph {
    /// Parses an unweighted, undirected graph from a file of the form:
    ///
    /// ```text
    /// nodeA-someNodeB
    /// nodeA-c
    /// c-someNodeB
    /// ```
    ///
    /// Node names may be any alphabetic ASCII string. Edges are represented by '-'.
    /// Edges may appear more than once, though they will only be counted once.
    ///
    /// Note that in this format, nodes with no edges are unrepresentable. Something
    /// to fix once an AoC problem requires it.
    pub fn from_file(filename: &str) -> AocResult<Self> {
        let mut edgesets: Vec<HashSet<usize>> = Vec::new();
        let mut names = Vec::new();
        let mut name2node = HashMap::new();

        let file = File::open(filename)?;
        for line in io::BufReader::new(file).lines() {
            let edge_strings = line?.split('-').map(String::from).collect::<Vec<String>>();
            if edge_strings.len() != 2
                || !edge_strings
                    .iter()
                    .all(|v| v.chars().all(|c| c.is_ascii_alphabetic()))
            {
                return failure(format!("Malformed edge {:?} in input", edge_strings));
            }

            let mut edge_ids = [0, 0];

            for (i, name) in edge_strings.into_iter().enumerate() {
                if let Some(node) = name2node.get(&name) {
                    edge_ids[i] = *node;
                } else {
                    let node_id = name2node.len();
                    edge_ids[i] = node_id;
                    edgesets.push(HashSet::new());
                    names.push(name.to_owned());
                    name2node.insert(name.to_owned(), node_id);
                }
            }
            edgesets[edge_ids[0]].insert(edge_ids[1]);
            edgesets[edge_ids[1]].insert(edge_ids[0]);
        }
        let edges = edgesets
            .into_iter()
            .map(|s| Vec::from_iter(s.into_iter()))
            .collect();
        Ok(UnweightedUndirectedGraph {
            edges,
            name2node,
            names,
        })
    }

    pub fn neighbour_names(&self, node_name: &str) -> AocResult<Vec<&str>> {
        let node = self
            .name2node
            .get(node_name)
            .ok_or("No node with name {node_name}")?;
        Ok(self.edges[*node]
            .iter()
            .map(|v| self.names[*v].as_str())
            .collect())
    }
}
