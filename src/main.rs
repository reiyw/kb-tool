use counter::Counter;
use kb_tool::graph::KG;
use kb_tool::triple::{read_triples, TripleOrder};
use std::fs::{self, File};
use std::io::{stdout, BufWriter, Write};
use std::path::PathBuf;
use structopt::StructOpt;

// TODO: dev, test への分割
// TODO: 評価に使う補助ファイルの生成

#[derive(StructOpt, Debug)]
#[structopt(name = "kb-tool")]
enum Opt {
    /// Cut off at frequencies of entities and relations
    #[structopt(name = "cutoff")]
    CutOff {
        /// File to process
        #[structopt(name = "FILE", parse(from_os_str))]
        file: PathBuf,
        /// Directory to store data
        #[structopt(long, parse(from_os_str), default_value = ".")]
        outdir: PathBuf,
        // /// Size ratio of processed to original data.
        // /// Must be between 0 and 1.
        // /// If specified, min_ent and min_rel are ignored
        // #[structopt(long)]
        // data_ratio: Option<f64>,
        /// Minimum count of entities
        #[structopt(long, default_value = "1")]
        min_ent: usize,
        /// Minimum count of relations
        #[structopt(long, default_value = "1")]
        min_rel: usize,
        /// Drop duplicated triples before to count
        #[structopt(long)]
        dedup_before_count: bool,
        /// Drop duplicated triples after to count
        #[structopt(long)]
        dedup_after_count: bool,
    },
    /// Sample paths
    #[structopt(name = "sample-path")]
    SamplePath {
        /// Sampling source
        #[structopt(name = "FILE", parse(from_os_str))]
        file: PathBuf,
        // /// Directory to store data
        // #[structopt(long, parse(from_os_str), default_value = ".")]
        // outdir: PathBuf,
        /// Length of a path
        #[structopt(long, default_value = "2")]
        path_len: usize,
        /// Maximum sample size
        #[structopt(long, default_value = "1000")]
        sample_size: usize,
        /// Drop duplicated paths
        #[structopt(long)]
        dedup: bool,
    },
}

fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    match opt {
        Opt::CutOff {
            file,
            outdir,
            min_ent,
            min_rel,
            dedup_before_count,
            dedup_after_count,
        } => cutoff_at_frequency(
            file,
            outdir,
            min_ent,
            min_rel,
            dedup_before_count,
            dedup_after_count,
        ),
        Opt::SamplePath {
            file,
            path_len,
            sample_size,
            dedup,
        } => sample_path(file, path_len, sample_size, dedup),
    }
}

fn cutoff_at_frequency(
    file: PathBuf,
    outdir: PathBuf,
    min_ent: usize,
    min_rel: usize,
    dedup_before_count: bool,
    dedup_after_count: bool,
) -> Result<(), std::io::Error> {
    let content = fs::read_to_string(file)?;
    let mut triples = read_triples(&content, TripleOrder::HRT);

    if dedup_before_count {
        triples.sort_unstable();
        triples.dedup();
    }

    let mut ents = Counter::<&str>::new();
    let mut rels = Counter::<&str>::new();
    for t in &triples {
        ents += vec![t.head, t.tail];
        rels += vec![t.relation];
    }

    let mut out = BufWriter::new(File::create(outdir.join("entity.vocab"))?);
    for (ent, c) in ents.most_common() {
        if c < min_ent {
            break;
        }
        writeln!(out, "{}\t{:.1}", ent, c as f64)?;
    }

    let mut out = BufWriter::new(File::create(outdir.join("relation.vocab"))?);
    for (rel, c) in rels.most_common() {
        if c < min_rel {
            break;
        }
        writeln!(out, "{}\t{:.1}", rel, c as f64)?;
    }

    if dedup_after_count {
        triples.sort_unstable();
        triples.dedup();
    }

    let mut out = BufWriter::new(File::create(outdir.join("train.txt"))?);
    for t in &triples {
        if ents[t.head] >= min_ent && ents[t.tail] >= min_ent && rels[t.relation] >= min_rel {
            writeln!(out, "{}\t{}\t{}", t.head, t.relation, t.tail)?;
        }
    }

    Ok(())
}

fn sample_path(
    file: PathBuf,
    path_len: usize,
    sample_size: usize,
    dedup: bool,
) -> Result<(), std::io::Error> {
    let content = fs::read_to_string(file)?;
    let triples = read_triples(&content, TripleOrder::HRT);
    let kg = KG::from_triples(triples);
    let mut rng = rand::thread_rng();
    let mut paths: Vec<_> = (0..sample_size)
        .map(|_| kg.sample_path(path_len, &mut rng))
        .collect();

    if dedup {
        paths.sort_unstable();
        paths.dedup();
    }

    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    for path in paths {
        writeln!(out, "{}", path)?;
    }

    Ok(())
}
