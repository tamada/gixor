use crate::{GixorError, Result};

pub(super) fn errs_vec_to_result<T>(errs: Vec<GixorError>, ok_result: T) -> Result<T> {
    if errs.is_empty() {
        Ok(ok_result)
    } else {
        single_err_or_errs_array(errs)
    }
}

pub(super) fn single_err_or_errs_array<T>(errs: Vec<GixorError>) -> Result<T> {
    if errs.len() == 1 {
        Err(errs.into_iter().next().unwrap())
    } else {
        Err(GixorError::Array(errs))
    }
}
