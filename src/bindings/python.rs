use std::collections::HashMap;

use crate::service::{FlagService, FlagServiceOptions};
use pyo3::prelude::*;

#[pyclass]
struct PythonFlagService {
    flag_service: FlagService,
}

#[pyclass]
struct PythonFlagOptions {
    retriever_data: Option<String>,
}

#[pymethods]
impl PythonFlagOptions {
    #[new]
    fn new(retriever_data: Option<String>) -> PyResult<PythonFlagOptions> {
        Ok(PythonFlagOptions {
            retriever_data: retriever_data,
        })
    }
}

#[pymethods]
impl PythonFlagService {
    #[new]
    fn new(options: &PythonFlagOptions) -> PyResult<PythonFlagService> {
        Ok(PythonFlagService {
            flag_service: FlagService::new(FlagServiceOptions {
                retriever_type: Some(crate::service::types::FlagRetrieverType::JSON),
                retriever_url: None,
                refresh_interval: 0,
                retriever_data: options.retriever_data.clone(),
            }),
        })
    }

    pub fn enabled(
        &self,
        name: &str,
        default: bool,
        context: Option<HashMap<String, String>>,
    ) -> PyResult<bool> {
        Ok(self.flag_service.enabled(name, default, context))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn mrflagly(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<PythonFlagService>()?;
    m.add_class::<PythonFlagOptions>()?;
    // m.add_class(PythonFlagService);
    Ok(())
}
