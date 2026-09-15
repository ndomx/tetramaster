#[macro_export]
macro_rules! min {
    ($x:expr) => {
        $x
    };

    ($x:expr, $($rest:expr),+) => {
        $x.min(min!($($rest),+))
    };
}