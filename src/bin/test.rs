use petgraph::graphmap::DiGraphMap;
use petgraph::Graph;

fn main() {
    let mut g = Graph::<&str, &str>::new();
    let pg = g.add_node("petgraph");
    let fb = g.add_node("fixedbitset");
    let qc = g.add_node("quickcheck");
    let rand = g.add_node("rand");
    let libc = g.add_node("libc");
    g.extend_with_edges(&[(pg, fb), (pg, qc), (qc, rand), (rand, libc), (qc, libc)]);
    g.add_edge(pg, fb, "weight1");
    println!("Hello, world!");
    println!("{:?}", g);

    for edge in g.edges(pg) {
        println!("{:?}", edge);
    }

    let mut dgm = DiGraphMap::<&str, &str>::new();
    dgm.add_edge("hoge", "fuga", "buri");
    dgm.add_edge("hoge", "hage", "hige");
    dgm.add_edge("hoge", "fuga", "hage");
    for edge in dgm.edges("hoge") {
        println!("{:?}", edge);
    }
}
