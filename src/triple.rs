// Don't generate String for each triple to avoid too much allocation.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Triple<'a> {
    pub head: &'a str,
    pub relation: &'a str,
    pub tail: &'a str,
}

impl<'a> Triple<'a> {
    pub fn new(head: &'a str, relation: &'a str, tail: &'a str) -> Self {
        Triple {
            head,
            relation,
            tail,
        }
    }
}

pub type Triples<'a> = Vec<Triple<'a>>;

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
