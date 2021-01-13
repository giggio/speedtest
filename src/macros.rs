macro_rules! printlnv {
        ($($arg:tt)*) => ({
            unsafe {
                if $crate::VERBOSE {
                    println!($($arg)*);
                }
            }
        })
    }
