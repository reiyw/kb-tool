use crate::triple::Triples;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug)]
struct Edge<'a> {
    id: usize,
    src: usize,
    dst: usize,
    label: &'a str,
}

#[derive(Debug)]
struct Node<'a> {
    id: usize,
    edges_fwd: Vec<usize>,
    edges_rev: Vec<usize>,
    label: &'a str,
}

impl<'a> Node<'a> {
    fn new(label: &'a str, id: usize) -> Self {
        Node {
            id,
            edges_fwd: Vec::new(),
            edges_rev: Vec::new(),
            label,
        }
    }
}

#[derive(Debug)]
pub struct KG<'a> {
    node_ids: HashMap<&'a str, usize>,
    nodes: Vec<Node<'a>>,
    edges: Vec<Edge<'a>>,
}

impl<'a> KG<'a> {
    pub fn from_triples(triples: Triples<'a>) -> Self {
        let mut node_ids = HashMap::new();
        let mut nodes: Vec<Node> = Vec::new();
        let mut edges = Vec::new();
        for tri in &triples {
            if !node_ids.contains_key(tri.head) {
                let id = nodes.len();
                node_ids.insert(tri.head, id);
                nodes.push(Node::new(tri.head, id));
            }
            let &i = node_ids.get(tri.head).unwrap();
            let head = nodes.get_mut(i).unwrap();
            head.edges_fwd.push(edges.len());

            if !node_ids.contains_key(tri.tail) {
                let id = nodes.len();
                node_ids.insert(tri.tail, id);
                nodes.push(Node::new(tri.tail, id));
            }
            let &j = node_ids.get(tri.tail).unwrap();
            let tail = nodes.get_mut(j).unwrap();
            tail.edges_rev.push(edges.len());

            edges.push(Edge {
                id: edges.len(),
                src: i,
                dst: j,
                label: tri.relation,
            });
        }
        KG {
            node_ids,
            nodes,
            edges,
        }
    }

    pub fn sample_path<R: Rng + ?Sized>(
        &self,
        path_len: usize,
        rng: &mut R,
        redge_suffix: &str,
        ledge_suffix: &str,
    ) -> String {
        let mut path = String::new();
        let mut node = self.select_node(rng);
        let mut prev_edge = None;
        path.push_str(node.label);
        for _ in 0..path_len {
            let (edge, fwd) = self.select_edge(node, prev_edge, rng);
            prev_edge = Some(edge);
            node = &self.nodes[if fwd { edge.dst } else { edge.src }];
            path += format!(
                "\t{}{}\t{}",
                edge.label,
                if fwd { redge_suffix } else { ledge_suffix },
                node.label
            )
            .as_str();
        }
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
        println!("{}", kg.sample_path(2, &mut rng, ">", "<"));
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
            println!("{}", kg.sample_path(2, &mut rng, "::-->", "::<--"));
        }
    }
}
