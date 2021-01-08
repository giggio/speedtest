#![macro_use]

macro_rules! printlnv {
        ($($arg:tt)*) => ({
            unsafe {
                if VERBOSE {
                    println!($($arg)*);
                }
            }
        })
    }
