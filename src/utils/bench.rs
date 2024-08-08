use std::arch::asm;

#[allow(unused, asm_sub_register)]
pub fn better_black_box(mut x: u32) -> u32 {
    unsafe {
        asm!("/* {x} */", x = inout(reg) x, options(nostack));
    }
    x
}
