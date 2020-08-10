use crate::triple::Triples;
use ndarray::prelude::*;
use ndarray::{Array, Ix1, Ix2};
use rand::Rng;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct Edge {
    id: usize,
    src: usize,
    dst: usize,
    label: String,
}

impl Edge {
    fn new(id: usize, src: usize, dst: usize, label: impl Into<String>) -> Self {
        Edge {
            id,
            src,
            dst,
            label: label.into(),
        }
    }
}

#[derive(Debug)]
struct Node {
    id: usize,
    edges_fwd: Vec<usize>,
    edges_rev: Vec<usize>,
    label: String,
}

impl Node {
    fn new(label: impl Into<String>, id: usize) -> Self {
        Node {
            id,
            edges_fwd: Vec::new(),
            edges_rev: Vec::new(),
            label: label.into(),
        }
    }
}

#[derive(Debug)]
pub struct KG {
    node_ids: HashMap<String, usize>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    // adj_matrices: HashMap<String, Array<u32, Ix2>>,
    correct_candidates: HashMap<String, HashSet<usize>>,
}

impl KG {
    pub fn from_triples(triples: Triples) -> Self {
        let mut node_ids = HashMap::new();
        let mut nodes: Vec<Node> = Vec::new();
        let mut edges = Vec::new();
        for tri in &triples {
            if !node_ids.contains_key(&tri.head) {
                let id = nodes.len();
                node_ids.insert(tri.head.clone(), id);
                nodes.push(Node::new(&tri.head, id));
            }
            let &i = node_ids.get(&tri.head).unwrap();
            let head = nodes.get_mut(i).unwrap();
            head.edges_fwd.push(edges.len());

            if !node_ids.contains_key(&tri.tail) {
                let id = nodes.len();
                node_ids.insert(tri.tail.clone(), id);
                nodes.push(Node::new(&tri.tail, id));
            }
            let &j = node_ids.get(&tri.tail).unwrap();
            let tail = nodes.get_mut(j).unwrap();
            tail.edges_rev.push(edges.len());

            edges.push(Edge::new(edges.len(), i, j, &tri.relation));
        }

        // let mut adj_matrices: HashMap<_, Array<u32, Ix2>> = HashMap::new();
        let mut correct_candidates = HashMap::new();
        for node in &nodes {
            for &edge_i in &node.edges_fwd {
                let edge = &edges[edge_i];
                let edge_label = format!("{}::-->", edge.label);

                // let mat = adj_matrices
                //     .entry(edge_label.clone())
                //     .or_insert(Array::zeros((nodes.len(), nodes.len())));
                // mat.slice_mut(s![edge.src, edge.dst]).fill(1);

                let tails = correct_candidates
                    .entry(edge_label.clone())
                    .or_insert(HashSet::new());
                tails.insert(edge.dst);
            }
            for &edge_i in &node.edges_rev {
                let edge = &edges[edge_i];
                let edge_label = format!("{}::<--", edge.label);

                // let mat = adj_matrices
                //     .entry(edge_label.clone())
                //     .or_insert(Array::zeros((nodes.len(), nodes.len())));
                // mat.slice_mut(s![edge.dst, edge.src]).fill(1);

                let tails = correct_candidates
                    .entry(edge_label.clone())
                    .or_insert(HashSet::new());
                tails.insert(edge.src);
            }
        }

        KG {
            node_ids,
            nodes,
            edges,
            // adj_matrices,
            correct_candidates,
        }
    }

    pub fn sample_path<R: Rng + ?Sized>(
        &self,
        path_len: usize,
        rng: &mut R,
        redge_suffix: &str,
        ledge_suffix: &str,
    ) -> Vec<String> {
        let mut path = Vec::new();
        let mut node = self.select_node(rng);
        let mut prev_edge = None;
        path.push(node.label.clone());
        for _ in 0..path_len {
            let (edge, fwd) = self.select_edge(node, prev_edge, rng);
            prev_edge = Some(edge);
            node = &self.nodes[if fwd { edge.dst } else { edge.src }];
            path.push(edge.label.clone() + if fwd { redge_suffix } else { ledge_suffix });
            // path.push(node.label);
        }
        path.push(node.label.clone());
        path
    }

    fn select_node<R: Rng + ?Sized>(&self, rng: &mut R) -> &Node {
        &self.nodes[rng.gen_range(0, self.nodes.len())]
    }

    /// Select randomly an edge connecting to the given node.
    ///
    /// Try to avoid selecting `prev_edge` as much as possible.
    fn select_edge<R: Rng + ?Sized>(
        &self,
        from: &Node,
        prev_edge: Option<&Edge>,
        rng: &mut R,
    ) -> (&Edge, bool) {
        let mut edges = [&from.edges_fwd[..], &from.edges_rev[..]].concat();
        let mut boundary = from.edges_fwd.len();
        if let Some(prev) = prev_edge {
            if let Some(i) = edges.iter().position(|&v| v == prev.id) {
                if edges.len() > 1 {
                    // remove `prev_edge` but ensure `edges.len() >= 1`
                    edges.remove(i);
                    if i < boundary {
                        boundary -= 1;
                    }
                }
            }
        };
        let i = rng.gen_range(0, edges.len());
        (&self.edges[edges[i]], i < boundary)
    }

    pub fn sample_negative_tail<R: Rng + ?Sized>(
        &self,
        path: &Vec<String>,
        rng: &mut R,
    ) -> Option<String> {
        let head = &path[0];
        let tail_i = self.node_ids[&path[path.len() - 1]];
        let candidates = &self.correct_candidates[&path[path.len() - 2]];

        // 実際に辿る処理が重かったので応急処置的に
        let candidates: Vec<usize> = candidates
            .iter()
            .filter(|&&i| i != tail_i)
            .cloned()
            .collect();

        // let i = self.node_ids[head];
        // let mut vec: Array<u32, Ix1> = Array::zeros(self.nodes.len());
        // vec.slice_mut(s![i]).fill(1);
        // for rel in &path[1..path.len() - 1] {
        //     let mat = &self.adj_matrices[rel];
        //     vec = vec.dot(mat);
        // }

        // // 実際に head から path を辿って到達できるノードを負例候補から削除
        // let actual_tails: HashSet<usize> = vec
        //     .indexed_iter()
        //     .filter_map(|(i, val)| if *val > 0 { Some(i) } else { None })
        //     .collect();
        // let candidates: Vec<usize> = candidates.difference(&actual_tails).cloned().collect();

        if candidates.len() == 0 {
            // None
            Some(self.nodes[rng.gen_range(0, self.nodes.len())].label.clone())
        } else {
            let i = rng.gen_range(0, candidates.len());
            Some(self.nodes[candidates[i]].label.clone())
        }

        // Some("".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triple::{read_triples, Triple, TripleOrder};
    use std::fs;

    #[test]
    fn sample_path() {
        let content = fs::read_to_string("data/sample.txt").unwrap();
        let triples = read_triples(&content, TripleOrder::HRT);
        let kg = KG::from_triples(triples);
        let mut rng = rand::thread_rng();
        println!("{:?}", kg.sample_path(2, &mut rng, ">", "<"));
    }

    #[test]
    fn regular_graph() {
        let triples = vec![
            Triple::new("A", "r1", "B"),
            Triple::new("B", "r2", "C"),
            Triple::new("C", "r3", "A"),
        ];
        let kg = KG::from_triples(triples);
        println!("{:#?}", kg);
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            println!("{:?}", kg.sample_path(2, &mut rng, "::-->", "::<--"));
        }
    }
}
