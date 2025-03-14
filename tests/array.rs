use ndarray::*;
use numpy::*;
use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyDict, PyList},
};

fn get_np_locals(py: Python) -> &PyDict {
    [("np", get_array_module(py).unwrap())].into_py_dict(py)
}

fn not_contiguous_array(py: Python) -> &PyArray1<i32> {
    py.eval(
        "np.array([1, 2, 3, 4], dtype='int32')[::2]",
        Some(get_np_locals(py)),
        None,
    )
    .unwrap()
    .downcast()
    .unwrap()
}

#[test]
fn new_c_order() {
    let dim = [3, 5];
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<f64, _>::zeros(py, dim, false);
        assert!(arr.ndim() == 2);
        assert!(arr.dims() == dim);
        let size = std::mem::size_of::<f64>() as isize;
        assert!(arr.strides() == [dim[1] as isize * size, size]);
    })
}

#[test]
fn new_fortran_order() {
    let dim = [3, 5];
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<f64, _>::zeros(py, dim, true);
        assert!(arr.ndim() == 2);
        assert!(arr.dims() == dim);
        let size = std::mem::size_of::<f64>() as isize;
        assert!(arr.strides() == [size, dim[0] as isize * size],);
    })
}

#[test]
fn tuple_as_dim() {
    let dim = (3, 5);
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<f64, _>::zeros(py, dim, false);
        assert!(arr.ndim() == 2);
        assert!(arr.dims() == [3, 5]);
    })
}

#[test]
fn zeros() {
    let shape = [3, 4];
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<f64, _>::zeros(py, shape, false);
        assert!(arr.ndim() == 2);
        assert!(arr.dims() == shape);
        assert!(arr.strides() == [shape[1] as isize * 8, 8]);

        let arr = PyArray::<f64, _>::zeros(py, shape, true);
        assert!(arr.ndim() == 2);
        assert!(arr.dims() == shape);
        assert!(arr.strides() == [8, shape[0] as isize * 8]);
    })
}

#[test]
fn arange() {
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<f64, _>::arange(py, 0.0, 1.0, 0.1);
        assert_eq!(arr.ndim(), 1);
        assert_eq!(arr.dims(), ndarray::Dim([10]));
    })
}

#[test]
fn as_array() {
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<f64, _>::zeros(py, [3, 2, 4], false);
        let arr = arr.readonly();
        let a = arr.as_array();
        assert_eq!(arr.shape(), a.shape());
        assert_eq!(
            arr.strides().iter().map(|x| x / 8).collect::<Vec<_>>(),
            a.strides()
        );
        let not_contiguous = not_contiguous_array(py).readonly();
        assert_eq!(not_contiguous.as_array(), array![1, 3]);
    })
}

#[test]
fn as_raw_array() {
    pyo3::Python::with_gil(|py| {
        let not_contiguous = not_contiguous_array(py);
        let raw_array_view = not_contiguous.as_raw_array();
        assert_eq!(unsafe { raw_array_view.deref_into_view()[0] }, 1);
        let raw_array_view_mut = not_contiguous.as_raw_array_mut();
        assert_eq!(unsafe { raw_array_view_mut.deref_into_view_mut()[1] }, 3);
    })
}

#[test]
fn as_slice() {
    pyo3::Python::with_gil(|py| {
        let arr = PyArray::<i32, _>::zeros(py, [3, 2, 4], false).readonly();
        assert_eq!(arr.as_slice().unwrap().len(), 3 * 2 * 4);
        let not_contiguous = not_contiguous_array(py).readonly();
        assert!(not_contiguous.as_slice().is_err());
    })
}

#[test]
fn is_instance() {
    pyo3::Python::with_gil(|py| {
        let arr = PyArray2::<f64>::zeros(py, [3, 5], false);
        assert!(arr.is_instance::<PyArray2<f64>>().unwrap());
        assert!(!arr.is_instance::<PyList>().unwrap());
    })
}

#[test]
fn from_vec2() {
    let vec2 = vec![vec![1, 2, 3]; 2];
    pyo3::Python::with_gil(|py| {
        let pyarray = PyArray::from_vec2(py, &vec2).unwrap();
        assert_eq!(pyarray.readonly().as_array(), array![[1, 2, 3], [1, 2, 3]]);
        assert!(PyArray::from_vec2(py, &[vec![1], vec![2, 3]]).is_err());
    })
}

#[test]
fn from_vec3() {
    let vec3 = vec![vec![vec![1, 2]; 2]; 2];
    pyo3::Python::with_gil(|py| {
        let pyarray = PyArray::from_vec3(py, &vec3).unwrap();
        assert_eq!(
            pyarray.readonly().as_array(),
            array![[[1, 2], [1, 2]], [[1, 2], [1, 2]]]
        );
    })
}

#[test]
fn from_eval_to_fixed() {
    pyo3::Python::with_gil(|py| {
        let locals = get_np_locals(py);
        let pyarray: &PyArray1<i32> = py
            .eval("np.array([1, 2, 3], dtype='int32')", Some(locals), None)
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(pyarray.readonly().as_array(), array![1, 2, 3]);
    })
}

#[test]
fn from_eval_to_dyn() {
    pyo3::Python::with_gil(|py| {
        let locals = get_np_locals(py);
        let pyarray: &PyArrayDyn<i32> = py
            .eval(
                "np.array([[1, 2], [3, 4]], dtype='int32')",
                Some(locals),
                None,
            )
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(
            pyarray.readonly().as_array(),
            array![[1, 2], [3, 4]].into_dyn()
        );
    })
}

#[test]
fn from_eval_to_dyn_u64() {
    pyo3::Python::with_gil(|py| {
        let locals = get_np_locals(py);
        let pyarray: &PyArrayDyn<u64> = py
            .eval(
                "np.array([[1, 2], [3, 4]], dtype='uint64')",
                Some(locals),
                None,
            )
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(
            pyarray.readonly().as_array(),
            array![[1, 2], [3, 4]].into_dyn()
        );
    })
}

#[test]
fn from_eval_fail_by_dtype() {
    pyo3::Python::with_gil(|py| {
        let locals = get_np_locals(py);
        let converted: Result<&PyArray1<i32>, _> = py
            .eval("np.array([1, 2, 3], dtype='float64')", Some(locals), None)
            .unwrap()
            .extract();
        converted.unwrap_err().print_and_set_sys_last_vars(py);
    })
}

#[test]
fn from_eval_fail_by_dim() {
    pyo3::Python::with_gil(|py| {
        let locals = get_np_locals(py);
        let converted: Result<&PyArray2<i32>, _> = py
            .eval("np.array([1, 2, 3], dtype='int32')", Some(locals), None)
            .unwrap()
            .extract();
        converted.unwrap_err().print_and_set_sys_last_vars(py);
    })
}

#[test]
fn array_cast() {
    let vec2 = vec![vec![1.0, 2.0, 3.0]; 2];
    pyo3::Python::with_gil(|py| {
        let arr_f64 = PyArray::from_vec2(py, &vec2).unwrap();
        let arr_i32: &PyArray2<i32> = arr_f64.cast(false).unwrap();
        assert_eq!(arr_i32.readonly().as_array(), array![[1, 2, 3], [1, 2, 3]]);
    })
}

#[test]
fn handle_negative_strides() {
    let arr = array![[2, 3], [4, 5u32]];
    pyo3::Python::with_gil(|py| {
        let pyarr = arr.to_pyarray(py);
        let negstr_pyarr: &numpy::PyArray2<u32> = py
            .eval("a[::-1]", Some([("a", pyarr)].into_py_dict(py)), None)
            .unwrap()
            .downcast()
            .unwrap();
        assert_eq!(negstr_pyarr.to_owned_array(), arr.slice(s![..;-1, ..]));
    })
}

#[test]
fn dtype_from_py() {
    pyo3::Python::with_gil(|py| {
        let arr = array![[2, 3], [4, 5u32]];
        let pyarr = arr.to_pyarray(py);
        let dtype: &numpy::PyArrayDescr = py
            .eval("a.dtype", Some([("a", pyarr)].into_py_dict(py)), None)
            .unwrap()
            .downcast()
            .unwrap();
        assert_eq!(&format!("{:?}", dtype), "dtype('uint32')");
        assert!(dtype.is_equiv_to(numpy::dtype::<u32>(py)));
    })
}

#[test]
fn borrow_from_array() {
    use numpy::ndarray::Array1;
    use pyo3::py_run;

    #[pyclass]
    struct Owner {
        array: Array1<f64>,
    }

    #[pymethods]
    impl Owner {
        #[getter]
        fn array(this: &PyCell<Self>) -> &PyArray1<f64> {
            let array = &this.borrow().array;

            unsafe { PyArray1::borrow_from_array(array, this) }
        }
    }

    let array = Python::with_gil(|py| {
        let owner = Py::new(
            py,
            Owner {
                array: Array1::linspace(0., 1., 10),
            },
        )
        .unwrap();

        owner.getattr(py, "array").unwrap()
    });

    Python::with_gil(|py| {
        py_run!(py, array, "assert array.shape == (10,)");
    });
}

#[test]
fn downcasting_works() {
    Python::with_gil(|py| {
        let ob: &PyAny = PyArray::from_slice(py, &[1_i32, 2, 3]);

        assert!(ob.downcast::<PyArray1<i32>>().is_ok());
    })
}

#[test]
fn downcasting_respects_element_type() {
    Python::with_gil(|py| {
        let ob: &PyAny = PyArray::from_slice(py, &[1_i32, 2, 3]);

        assert!(ob.downcast::<PyArray1<f64>>().is_err());
    })
}

#[test]
fn downcasting_respects_dimensionality() {
    Python::with_gil(|py| {
        let ob: &PyAny = PyArray::from_slice(py, &[1_i32, 2, 3]);

        assert!(ob.downcast::<PyArray2<i32>>().is_err());
    })
}
