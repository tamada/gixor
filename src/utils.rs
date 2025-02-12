use std::path::Path;

use crate::{GibbsError, Result};

pub(crate) fn errs_vec_to_result<T>(errs: Vec<GibbsError>, ok_result: T) -> Result<T> {
    if errs.is_empty() {
        Ok(ok_result)
    } else if errs.len() == 1{
        Err(errs.into_iter().next().unwrap())
    } else {
        Err(GibbsError::Array(errs))
    }
}

pub(crate) fn has_extension<S: AsRef<str>>(path: &Path, ext: S) -> bool {
    path.extension().map_or(false, |e| e == ext.as_ref())
}

