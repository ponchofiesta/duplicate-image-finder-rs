#[macro_export]
macro_rules! debug {
    () => {
        println!()
    };
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }};
}
pub use debug;