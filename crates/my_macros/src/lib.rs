/// Attempt to match a series of bindings against patterns and return the first match.
///
/// # Example
/// ```
/// use my_macros::first_match;
///
/// let first_option: Option<i32> = None;
/// let second_option: Option<i32> = None;
/// let third_option = Some(42);
/// let fourth_option = Some(1234);
///
/// let res = first_match! {
///     Some(first) = first_option => unreachable!("Doesn't match"),
///     Some(second) = second_option => unreachable!("Doesn't match"),
///     Some(third) = third_option => third,
///     Some(fourth) = fourth_option => unreachable!("Matches, but comes after `third`"),
/// };
///
/// assert_eq!(res, Some(42));
/// ```
#[macro_export]
macro_rules! first_match {
    // Base case: The final arm (no trailing comma required)
    ($pat:pat = $expr:expr => $body:expr $(,)?) => {
        if let $pat = $expr {
            ::core::option::Option::Some($body)
        } else {
            ::core::option::Option::None
        }
    };

    // Recursive case: Take the first arm, recurse on the rest
    ($pat:pat = $expr:expr => $body:expr, $($rest:tt)+) => {
        if let $pat = $expr {
            ::core::option::Option::Some($body)
        } else {
            $crate::first_match!($($rest)+)
        }
    };
}
