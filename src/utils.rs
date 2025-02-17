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

/// Convert `Vec<Result<T>>` to `Result<Vec<T>>`
/// If `Vec<Result<T>>` has the multiple errors,
/// `Result<Vec<T>>` returns `Err(GixorError::Array(Vec<GixorError>))`.
pub(super) fn vec_result_to_result_vec<T>(result: Vec<Result<T>>) -> Result<Vec<T>> {
    let mut errs = vec![];
    let mut ok_results = vec![];
    for r in result {
        match r {
            Ok(ok) => ok_results.push(ok),
            Err(err) => errs.push(err),
        }
    }
    errs_vec_to_result(errs, ok_results)
}
