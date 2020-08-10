mod graph;
mod triple;

use crate::graph::KG;
use crate::triple::{read_triples, TripleOrder};
use pyo3::prelude::*;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Poisson};
use std::convert::TryInto;
use std::{cmp, fs};

#[pyclass]
#[derive(Debug)]
struct PathSampler {
    data_path: String,
    mean_path_len: f64,
    max_path_len: usize,
    poi: Poisson<f64>,
    rng: StdRng,
    kg: KG,
    data_size: usize,
}

#[pymethods]
impl PathSampler {
    #[allow(clippy::new_ret_no_self)]
    #[new]
    fn new(
        obj: &PyRawObject,
        data_path: String,
        mean_path_len: f64,
        max_path_len: usize,
        random_state: u64,
    ) {
        let content = fs::read_to_string(&data_path).unwrap();
        let triples = read_triples(&content, TripleOrder::HRT);
        let data_size = triples.len();
        let kg = KG::from_triples(triples);

        let mut bytes: Vec<u8> = random_state.to_be_bytes().to_vec();
        bytes.extend(vec![0; 24]);
        let seed: [u8; 32] = (&bytes[..])
            .try_into()
            .expect("slice with incorrect length");
        let poi = Poisson::new(mean_path_len - 0.0).unwrap();
        let rng = rand::SeedableRng::from_seed(seed);

        obj.init({
            PathSampler {
                data_path,
                mean_path_len,
                max_path_len,
                poi,
                rng,
                kg,
                data_size,
            }
        });
    }

    #[getter]
    fn data_size(&self) -> PyResult<usize> {
        Ok(self.data_size)
    }

    fn sample_path(&mut self, _py: Python) -> PyResult<Vec<String>> {
        let v: f64 = self.poi.sample(&mut self.rng);
        let path_len = cmp::min((v + 1.0) as usize, self.max_path_len);
        Ok(self
            .kg
            .sample_path(path_len, &mut self.rng, "::-->", "::<--"))
    }

    fn sample_path_with_negative(&mut self, _py: Python) -> PyResult<(Vec<String>, String)> {
        let v: f64 = self.poi.sample(&mut self.rng);
        let path_len = cmp::min((v + 1.0) as usize, self.max_path_len);
        let path = self
            .kg
            .sample_path(path_len, &mut self.rng, "::-->", "::<--");
        let negative_tail = self.kg.sample_negative_tail(&path, &mut self.rng);
        Ok((path, negative_tail.unwrap_or("".into())))
    }
}

#[pymodule]
fn kb_tool(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PathSampler>()?;

    Ok(())
}
