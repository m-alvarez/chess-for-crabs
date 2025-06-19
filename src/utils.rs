use std::arch::asm;
use std::num::Wrapping;

#[macro_export]
macro_rules! const_for {
    ($var:ident in $range:expr => $body:stmt) => {
        {
            assert!($range.start <= $range.end);
            let mut $var = $range.start;
            while $var < $range.end {
                $body
                $var = $var + 1
            }
        }
    };
}

#[macro_export]
macro_rules! const_foreach {
    ($var:pat in $arr:expr => $body:stmt) => {{
        let mut i = 0;
        while i < $arr.len() {
            let $var = $arr[i];
            i += 1;
            $body
        }
    }};
}

pub fn lsb(b: u64) -> u64 {
    let result = b & (Wrapping(!b) + Wrapping(1)).0;
    result
}

// Just don't play chess on ARM, it's that simple
pub fn msb(b: u64) -> u64 {
    let mut result: u64 = 1 << 63;
    unsafe {
        asm!(
            "lzcnt rcx, {b}",
            "shr {result}, cl",
            "and {result}, {b}", // Haha
            b = in(reg) b,
            result = inout(reg) result,
            out("rcx") _,
        )
    }
    result
}
