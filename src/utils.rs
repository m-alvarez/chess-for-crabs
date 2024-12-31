use std::num::Wrapping;
use std::arch::asm;

#[macro_export]
macro_rules! const_for {
    ($var:ident in $range:expr => $body:stmt) => {
        {
            let mut $var = $range.start;
            while $var < $range.end {
                $body
                $var += 1
            }
        }
    }
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
            b = in(reg) b,
            result = inout(reg) result,
            out("rcx") _,
        )
    }
    result
}
