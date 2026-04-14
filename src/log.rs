#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)+) => {{
        println!(
            "[{}] {}: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            $level,
            format!($($arg)+)
        );
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)+) => {
        crate::log!("INFO", $($arg)+);
    };
}