use polars::prelude::*;
use pyo3::exceptions::RuntimeError;
use pyo3::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PyPolarsEr {
    #[error(transparent)]
    Any(#[from] PolarsError),
    #[error("{0}")]
    Other(String),
}

impl std::convert::From<PyPolarsEr> for PyErr {
    fn from(err: PyPolarsEr) -> PyErr {
        RuntimeError::py_err(format!("{:?}", err))
    }
}

#[pyclass]
#[repr(transparent)]
#[derive(Clone)]
pub struct PySeries {
    pub series: Series,
}

impl PySeries {
    fn new(series: Series) -> Self {
        PySeries { series }
    }
}

macro_rules! init_method {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            #[new]
            pub fn $name(name: &str, val: Vec<$type>) -> PySeries {
                PySeries {
                    series: Series::new(name, &val),
                }
            }
        }
    };
}

init_method!(new_i32, i32);
init_method!(new_i64, i64);
init_method!(new_f32, f32);
init_method!(new_f64, f64);
init_method!(new_bool, bool);
init_method!(new_u32, u32);
init_method!(new_date32, i32);
init_method!(new_date64, i64);
init_method!(new_duration_ns, i64);
init_method!(new_time_ns, i64);
init_method!(new_str, &str);

#[pymethods]
impl PySeries {
    pub fn name(&self) -> &str {
        self.series.name()
    }

    pub fn rename(&mut self, name: &str) {
        self.series.rename(name)
    }

    pub fn dtype(&self) -> &str {
        self.series.dtype().to_str()
    }

    pub fn n_chunks(&self) -> usize {
        self.series.n_chunks()
    }

    pub fn limit(&self, num_elements: usize) -> PyResult<Self> {
        let series = self.series.limit(num_elements).map_err(PyPolarsEr::from)?;
        Ok(PySeries { series })
    }

    pub fn slice(&self, offset: usize, length: usize) -> PyResult<Self> {
        let series = self
            .series
            .slice(offset, length)
            .map_err(PyPolarsEr::from)?;
        Ok(PySeries { series })
    }

    pub fn append(&mut self, other: &PySeries) -> PyResult<()> {
        self.series
            .append(&other.series)
            .map_err(PyPolarsEr::from)?;
        Ok(())
    }

    pub fn filter(&self, filter: &PySeries) -> PyResult<Self> {
        let filter_series = &filter.series;
        if let Series::Bool(ca) = filter_series {
            let series = self.series.filter(ca).map_err(PyPolarsEr::from)?;
            Ok(PySeries { series })
        } else {
            Err(RuntimeError::py_err("Expected a boolean mask"))
        }
    }

    pub fn add(&self, other: &PySeries) -> PyResult<Self> {
        Ok(PySeries::new(&self.series + &other.series))
    }

    pub fn sub(&self, other: &PySeries) -> PyResult<Self> {
        Ok(PySeries::new(&self.series - &other.series))
    }

    pub fn mul(&self, other: &PySeries) -> PyResult<Self> {
        Ok(PySeries::new(&self.series * &other.series))
    }

    pub fn div(&self, other: &PySeries) -> PyResult<Self> {
        Ok(PySeries::new(&self.series / &other.series))
    }

    pub fn head(&self, length: Option<usize>) -> PyResult<Self> {
        Ok(PySeries::new(self.series.head(length)))
    }

    pub fn tail(&self, length: Option<usize>) -> PyResult<Self> {
        Ok(PySeries::new(self.series.tail(length)))
    }

    pub fn sort(&mut self) {
        self.series.sort();
    }

    pub fn argsort(&self) -> PyResult<Vec<usize>> {
        Ok(self.series.argsort())
    }

    pub fn arg_unique(&self) -> PyResult<Vec<usize>> {
        Ok(self.series.arg_unique())
    }

    pub fn take(&self, indices: Vec<usize>) -> PyResult<Self> {
        let take = self.series.take(&indices).map_err(PyPolarsEr::from)?;
        Ok(PySeries::new(take))
    }

    pub fn null_count(&self) -> PyResult<usize> {
        Ok(self.series.null_count())
    }

    pub fn is_null(&self) -> PySeries {
        todo!()
    }

    pub fn series_equal(&self, other: &PySeries) -> PyResult<bool> {
        Ok(self.series.series_equal(&other.series))
    }

    pub fn eq(&self, rhs: &PySeries) -> PyResult<Self> {
        Ok(Self::new(Series::Bool(self.series.eq(&rhs.series))))
    }

    pub fn neq(&self, rhs: &PySeries) -> PyResult<Self> {
        Ok(Self::new(Series::Bool(self.series.neq(&rhs.series))))
    }

    pub fn gt(&self, rhs: &PySeries) -> PyResult<Self> {
        Ok(Self::new(Series::Bool(self.series.gt(&rhs.series))))
    }

    pub fn gt_eq(&self, rhs: &PySeries) -> PyResult<Self> {
        Ok(Self::new(Series::Bool(self.series.gt_eq(&rhs.series))))
    }

    pub fn lt(&self, rhs: &PySeries) -> PyResult<Self> {
        Ok(Self::new(Series::Bool(self.series.lt(&rhs.series))))
    }

    pub fn lt_eq(&self, rhs: &PySeries) -> PyResult<Self> {
        Ok(Self::new(Series::Bool(self.series.lt_eq(&rhs.series))))
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.series))
    }
}

macro_rules! impl_arithmetic {
    ($name:ident, $type:ty, $operand:tt) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, other: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(&self.series $operand other))
            }
        }
    };
}

impl_arithmetic!(add_u32, u32, +);
impl_arithmetic!(add_i32, i32, +);
impl_arithmetic!(add_i64, i64, +);
impl_arithmetic!(add_f32, f32, +);
impl_arithmetic!(add_f64, f64, +);
impl_arithmetic!(sub_u32, u32, -);
impl_arithmetic!(sub_i32, i32, -);
impl_arithmetic!(sub_i64, i64, -);
impl_arithmetic!(sub_f32, f32, -);
impl_arithmetic!(sub_f64, f64, -);
impl_arithmetic!(div_u32, u32, /);
impl_arithmetic!(div_i32, i32, /);
impl_arithmetic!(div_i64, i64, /);
impl_arithmetic!(div_f32, f32, /);
impl_arithmetic!(div_f64, f64, /);
impl_arithmetic!(mul_u32, u32, *);
impl_arithmetic!(mul_i32, i32, *);
impl_arithmetic!(mul_i64, i64, *);
impl_arithmetic!(mul_f32, f32, *);
impl_arithmetic!(mul_f64, f64, *);

macro_rules! impl_rhs_arithmetic {
    ($name:ident, $type:ty, $operand:ident) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, other: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(other.$operand(&self.series)))
            }
        }
    };
}

impl_rhs_arithmetic!(add_u32_rhs, u32, add);
impl_rhs_arithmetic!(add_i32_rhs, i32, add);
impl_rhs_arithmetic!(add_i64_rhs, i64, add);
impl_rhs_arithmetic!(add_f32_rhs, f32, add);
impl_rhs_arithmetic!(add_f64_rhs, f64, add);
impl_rhs_arithmetic!(sub_u32_rhs, u32, sub);
impl_rhs_arithmetic!(sub_i32_rhs, i32, sub);
impl_rhs_arithmetic!(sub_i64_rhs, i64, sub);
impl_rhs_arithmetic!(sub_f32_rhs, f32, sub);
impl_rhs_arithmetic!(sub_f64_rhs, f64, sub);
impl_rhs_arithmetic!(div_u32_rhs, u32, div);
impl_rhs_arithmetic!(div_i32_rhs, i32, div);
impl_rhs_arithmetic!(div_i64_rhs, i64, div);
impl_rhs_arithmetic!(div_f32_rhs, f32, div);
impl_rhs_arithmetic!(div_f64_rhs, f64, div);
impl_rhs_arithmetic!(mul_u32_rhs, u32, mul);
impl_rhs_arithmetic!(mul_i32_rhs, i32, mul);
impl_rhs_arithmetic!(mul_i64_rhs, i64, mul);
impl_rhs_arithmetic!(mul_f32_rhs, f32, mul);
impl_rhs_arithmetic!(mul_f64_rhs, f64, mul);

macro_rules! impl_sum {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self) -> PyResult<Option<$type>> {
                Ok(self.series.sum())
            }
        }
    };
}

impl_sum!(sum_u32, u32);
impl_sum!(sum_i32, i32);
impl_sum!(sum_i64, i64);
impl_sum!(sum_f32, f32);
impl_sum!(sum_f64, f64);

macro_rules! impl_min {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self) -> PyResult<Option<$type>> {
                Ok(self.series.min())
            }
        }
    };
}

impl_min!(min_u32, u32);
impl_min!(min_i32, i32);
impl_min!(min_i64, i64);
impl_min!(min_f32, f32);
impl_min!(min_f64, f64);

macro_rules! impl_mean {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self) -> PyResult<Option<$type>> {
                Ok(self.series.mean())
            }
        }
    };
}

impl_mean!(mean_u32, u32);
impl_mean!(mean_i32, i32);
impl_mean!(mean_i64, i64);
impl_mean!(mean_f32, f32);
impl_mean!(mean_f64, f64);

macro_rules! impl_eq_num {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, rhs: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(Series::Bool(self.series.eq(rhs))))
            }
        }
    };
}

impl_eq_num!(eq_u32, u32);
impl_eq_num!(eq_i32, i32);
impl_eq_num!(eq_i64, i64);
impl_eq_num!(eq_f32, f32);
impl_eq_num!(eq_f64, f64);

macro_rules! impl_neq_num {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, rhs: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(Series::Bool(self.series.neq(rhs))))
            }
        }
    };
}

impl_neq_num!(neq_u32, u32);
impl_neq_num!(neq_i32, i32);
impl_neq_num!(neq_i64, i64);
impl_neq_num!(neq_f32, f32);
impl_neq_num!(neq_f64, f64);

macro_rules! impl_gt_num {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, rhs: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(Series::Bool(self.series.gt(rhs))))
            }
        }
    };
}

impl_gt_num!(gt_u32, u32);
impl_gt_num!(gt_i32, i32);
impl_gt_num!(gt_i64, i64);
impl_gt_num!(gt_f32, f32);
impl_gt_num!(gt_f64, f64);

macro_rules! impl_gt_eq_num {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, rhs: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(Series::Bool(self.series.gt_eq(rhs))))
            }
        }
    };
}

impl_gt_eq_num!(gt_eq_u32, u32);
impl_gt_eq_num!(gt_eq_i32, i32);
impl_gt_eq_num!(gt_eq_i64, i64);
impl_gt_eq_num!(gt_eq_f32, f32);
impl_gt_eq_num!(gt_eq_f64, f64);

macro_rules! impl_lt_num {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, rhs: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(Series::Bool(self.series.lt(rhs))))
            }
        }
    };
}

impl_lt_num!(lt_u32, u32);
impl_lt_num!(lt_i32, i32);
impl_lt_num!(lt_i64, i64);
impl_lt_num!(lt_f32, f32);
impl_lt_num!(lt_f64, f64);

macro_rules! impl_lt_eq_num {
    ($name:ident, $type:ty) => {
        #[pymethods]
        impl PySeries {
            pub fn $name(&self, rhs: $type) -> PyResult<PySeries> {
                Ok(PySeries::new(Series::Bool(self.series.lt_eq(rhs))))
            }
        }
    };
}

impl_lt_eq_num!(lt_eq_u32, u32);
impl_lt_eq_num!(lt_eq_i32, i32);
impl_lt_eq_num!(lt_eq_i64, i64);
impl_lt_eq_num!(lt_eq_f32, f32);
impl_lt_eq_num!(lt_eq_f64, f64);

fn to_series_collection(ps: Vec<PySeries>) -> Vec<Series> {
    // prevent destruction of ps
    let mut ps = std::mem::ManuallyDrop::new(ps);

    // get mutable pointer and reinterpret as Series
    let p = ps.as_mut_ptr() as *mut Series;
    let len = ps.len();
    let cap = ps.capacity();

    // The pointer ownership will be transferred to Vec and this will be responsible for dealoc
    unsafe { Vec::from_raw_parts(p, len, cap) }
}

fn to_pyseries_collection(s: Vec<Series>) -> Vec<PySeries> {
    let mut s = std::mem::ManuallyDrop::new(s);

    let p = s.as_mut_ptr() as *mut PySeries;
    let len = s.len();
    let cap = s.capacity();

    unsafe { Vec::from_raw_parts(p, len, cap) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn transmute_to_series() {
        // NOTE: This is only possible because PySeries is #[repr(transparent)]
        // https://doc.rust-lang.org/reference/type-layout.html
        let ps = PySeries {
            series: [1i32, 2, 3].iter().collect(),
        };

        let s = unsafe { std::mem::transmute::<PySeries, Series>(ps.clone()) };

        assert_eq!(s.sum::<i32>(), Some(6));
        let collection = vec![ps];
        let s = to_series_collection(collection);
        assert_eq!(
            s.iter().map(|s| s.sum::<i32>()).collect::<Vec<_>>(),
            vec![Some(6)]
        );
    }

    #[test]
    fn print() {
        let ps = PySeries {
            series: [1i32, 2, 3].iter().collect(),
        };
        let str = ps.__str__().unwrap();
        println!("{}", str);
        assert_eq!(
            str,
            "Series: i32
[
	1
	2
	3
]"
        )
    }
}
