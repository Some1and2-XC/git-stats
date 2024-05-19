/// Macro for continuing on errors
/// ```
/// # use anyhow::{Result, anyhow};
/// # mod git_stats::macros;
/// use macros::ok_or_continue;
///
/// fn divide_one(n: f32) -> Result<f32> {
///     if n == 0.0 {
///         return Err(anyhow!("Divide by zero error!"));
///     } else {
///         return Ok(1.0 / n);
///     }
/// }
///
/// let sum = 0.0;
/// for i in -4..5 {
///     sum += ok_or_continue!(divide_one(i.into()));
/// }
///
/// assert_eq!(sum, 1.0 / 5.0);
/// ```
macro_rules! ok_or_continue {
    ($res:expr) => {
        match $res {
            Ok(v) => v,
            Err(e) => {
                log::warn!("An error: {}, skipped.", e);
                continue;
            },
        }
    };
}
pub(crate) use ok_or_continue;
