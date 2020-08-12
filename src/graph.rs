use crate::triple::Triples;
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
    correct_candidates: HashMap<String, HashSet<usize>>,
    correct_hr2t: HashMap<(usize, String), HashSet<usize>>,
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

        let mut correct_candidates = HashMap::new();
        let mut correct_hr2t: HashMap<(usize, String), HashSet<usize>> = HashMap::new();

        for node in &nodes {
            for &edge_i in &node.edges_fwd {
                let edge = &edges[edge_i];
                let edge_label = format!("{}::-->", edge.label);

                let tails = correct_candidates
                    .entry(edge_label.clone())
                    .or_insert(HashSet::new());
                tails.insert(edge.dst);

                let tails = correct_hr2t
                    .entry((edge.src, edge_label.clone()))
                    .or_insert(HashSet::new());
                tails.insert(edge.dst);
            }
            for &edge_i in &node.edges_rev {
                let edge = &edges[edge_i];
                let edge_label = format!("{}::<--", edge.label);

                let tails = correct_candidates
                    .entry(edge_label.clone())
                    .or_insert(HashSet::new());
                tails.insert(edge.src);

                let tails = correct_hr2t
                    .entry((edge.dst, edge_label.clone()))
                    .or_insert(HashSet::new());
                tails.insert(edge.src);
            }
        }

        KG {
            node_ids,
            nodes,
            edges,
            correct_candidates,
            correct_hr2t,
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

    /// tail 以外のノードを uniform に選ぶ
    pub fn sample_negative_tail_uniformly<R: Rng + ?Sized>(
        &self,
        path: &Vec<String>,
        rng: &mut R,
    ) -> Option<String> {
        // あらかじめ一つ小さく生成しておく
        let mut i = rng.gen_range(0, self.nodes.len() - 1);

        let tail_i = self.node_ids[&path[path.len() - 1]];
        // tail_i 以上の index を 1 増やせば tail_i 自身はサンプリングされない
        if i >= tail_i {
            i += 1;
        }

        Some(self.nodes[i].label.clone())
    }

    /// Guu+'15 EMNLP っぽい negative sampling．single-hop でないとほぼ意味がない
    pub fn sample_negative_tail_near_miss<R: Rng + ?Sized>(
        &self,
        path: &Vec<String>,
        rng: &mut R,
    ) -> Option<String> {
        let head_i = self.node_ids[&path[0]];
        let tail_i = self.node_ids[&path[path.len() - 1]];
        let last_relation = &path[path.len() - 2];
        let type_matching_tails = &self.correct_candidates[last_relation];
        let actually_reachable_tails = &self.correct_hr2t[&(head_i, last_relation.clone())];
        let negs: Vec<usize> = type_matching_tails
            .difference(actually_reachable_tails)
            .cloned()
            .filter(|&i| i != tail_i) // query 自身は negative example にしない
            .collect();

        if negs.len() == 0 {
            // backoff to uniform sampling
            self.sample_negative_tail_uniformly(path, rng)
        } else {
            let i = rng.gen_range(0, negs.len());
            Some(self.nodes[negs[i]].label.clone())
        }
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
