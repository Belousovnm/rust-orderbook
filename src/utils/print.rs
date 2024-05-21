#[macro_export]
macro_rules! dbgp {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}
