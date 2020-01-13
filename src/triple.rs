#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Triple {
    pub head: String,
    pub relation: String,
    pub tail: String,
}

impl Triple {
    pub fn new(
        head: impl Into<String>,
        relation: impl Into<String>,
        tail: impl Into<String>,
    ) -> Self {
        Triple {
            head: head.into(),
            relation: relation.into(),
            tail: tail.into(),
        }
    }
}

pub type Triples = Vec<Triple>;

pub enum TripleOrder {
    HRT,
    HTR,
}

pub fn read_triples(triples: &str, order: TripleOrder) -> Triples {
    triples
        .lines()
        .map(|line| {
            let v: Vec<&str> = line.split('\t').collect();
            match &order {
                TripleOrder::HRT => Triple::new(v[0], v[1], v[2]),
                TripleOrder::HTR => Triple::new(v[0], v[2], v[1]),
            }
        })
        .collect()
}
