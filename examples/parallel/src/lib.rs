extern crate blas_src;

use ndarray::Zip;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1, PyReadonlyArray2};
use pyo3::{pymodule, types::PyModule, PyResult, Python};

#[pymodule]
fn rust_parallel(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    fn rows_dot<'py>(
        py: Python<'py>,
        x: PyReadonlyArray2<'py, f64>,
        y: PyReadonlyArray1<'py, f64>,
    ) -> &'py PyArray1<f64> {
        let x = x.as_array();
        let y = y.as_array();
        let z = Zip::from(x.rows()).par_map_collect(|row| row.dot(&y));
        z.into_pyarray(py)
    }
    Ok(())
}
