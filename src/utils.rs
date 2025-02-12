use crate::{GixorError, Result};

pub(crate) fn errs_vec_to_result<T>(errs: Vec<GixorError>, ok_result: T) -> Result<T> {
    if errs.is_empty() {
        Ok(ok_result)
    } else if errs.len() == 1 {
        Err(errs.into_iter().next().unwrap())
    } else {
        Err(GixorError::Array(errs))
    }
}
