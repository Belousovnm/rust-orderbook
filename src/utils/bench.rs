use std::arch::asm;

fn better_black_box(mut x: u64) -> u64 {
    unsafe {
        asm!("/* {x} */", x = inout(reg) x, options(nostack));
    }
    x
}
