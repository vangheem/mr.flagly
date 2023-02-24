use crate::service;
use crate::service::types;
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
struct FlagService {
    flag_service: service::FlagService,
}

#[pymethods]
impl FlagService {
    #[new]
    fn new(
        url: Option<String>,
        data: Option<String>,
        env_var: Option<String>,
        refresh_interval: Option<u64>,
    ) -> PyResult<FlagService> {
        let mut finder_type = types::FlagFinderType::NULL;
        if url.is_some() {
            finder_type = types::FlagFinderType::URL;
        } else if data.is_some() {
            finder_type = types::FlagFinderType::JSON;
        } else if env_var.is_some() {
            finder_type = types::FlagFinderType::ENVVAR;
        }

        let mut real_refresh_interval = 300;
        if refresh_interval.is_some() {
            real_refresh_interval = refresh_interval.unwrap();
        }
        Ok(FlagService {
            flag_service: service::FlagService::new(service::FlagServiceOptions {
                finder_type: finder_type,
                url: url,
                refresh_interval: real_refresh_interval,
                data: data,
                env_var: env_var,
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
    m.add_class::<FlagService>()?;
    // m.add_class(PythonFlagService);
    Ok(())
}
