/// Macro for continuing on errors
/// ```
/// # use std::error::Error;
/// # use git_stats::macros::ok_or_continue;
/// // Defines function that could error
/// fn error_on_even(v: usize) -> Result<usize, ()> {
///     if v % 2 == 0 {
///         return Err(());
///     } else {
///         return Ok(v);
///     }
/// }
///
/// // Should add 1 and 3
/// // and also just ignore the errors
/// let mut sum = 0;
/// for i in 0..4 {
///     sum += ok_or_continue!(error_on_even(i));
/// }
///
/// assert_eq!(sum, 4);
/// ```
#[doc(hidden)] #[macro_export]
macro_rules! __ok_or_continue {
    ($res:expr) => {
        match $res {
            Ok(v) => v,
            Err(e) => {
                log::warn!("An error: {:?}, skipped.", e);
                continue;
            },
        }
    };
}

/// Macro for continuing on errors
/// ```
/// # use std::error::Error;
/// # use git_stats::macros::ok_or_continue;
/// // Defines function that could error
/// fn error_on_even(v: usize) -> Result<usize, ()> {
///     if v % 2 == 0 {
///         return Err(());
///     } else {
///         return Ok(v);
///     }
/// }
///
/// // Should add 1 and 3
/// // and also just ignore the errors
/// let mut sum = 0;
/// for i in 0..4 {
///     sum += ok_or_continue!(error_on_even(i));
/// }
///
/// assert_eq!(sum, 4);
/// ```
#[doc(inline)]
pub use __ok_or_continue as ok_or_continue;


/// Allows clap to use enum variants as variants in CLI.
#[doc(hidden)] #[macro_export]
macro_rules! __clap_enum_variants {
    ($e: ty) => {{
        use clap::builder::TypedValueParser;
        clap::builder::PossibleValuesParser::new(
            <$e>::iter()
                .map(|v| {
                    v.to_string()
                })
                .collect::<Vec<String>>()
        )
        .map(|s| s.parse::<$e>().unwrap())
    }};
}

#[doc(inline)]
pub use __clap_enum_variants as clap_enum_variants;
