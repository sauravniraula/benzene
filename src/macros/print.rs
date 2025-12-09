#[macro_export]
macro_rules! print_dash {
    () => {
        println!("{}", "-".repeat(80));
    };
    ($length:expr) => {
        println!("{}", "-".repeat($length));
    };
    ($length:expr, $char:expr) => {
        println!("{}", $char.to_string().repeat($length));
    };
}

#[macro_export]
macro_rules! print_dash_no_newline {
    () => {
        print!("{}", "-".repeat(80));
    };
    ($length:expr) => {
        print!("{}", "-".repeat($length));
    };
    ($length:expr, $char:expr) => {
        print!("{}", $char.to_string().repeat($length));
    };
}

#[macro_export]
macro_rules! print_separator {
    ($text:expr) => {
        $crate::macros::print::print_separator_fn($text, 80, '-');
    };
    ($text:expr, $length:expr) => {
        $crate::macros::print::print_separator_fn($text, $length, '-');
    };
    ($text:expr, $length:expr, $char:expr) => {
        $crate::macros::print::print_separator_fn($text, $length, $char);
    };
}

#[doc(hidden)]
#[allow(unused)]
pub fn print_separator_fn(text: &str, length: usize, ch: char) {
    let separator_length = (length - text.len() - 2) / 2;
    let left_separator = ch.to_string().repeat(separator_length);
    let right_separator = ch.to_string().repeat(separator_length);
    println!("\n{} {} {}\n", left_separator, text, right_separator);
}

#[macro_export]
macro_rules! log {
    ($msg:expr) => {
        if $crate::constants::DEBUG {
            println!(
                "INFO[{}]: {}",
                chrono::Local::now().format("%Y/%m/%d %H:%M"),
                $msg
            );
        }
    };
    ($type:expr, $msg:expr) => {
        if $crate::constants::DEBUG {
            println!(
                "{}[{}]: {}",
                chrono::Local::now().format("%Y/%m/%d %H:%M"),
                $type,
                $msg
            );
        }
    };
}
