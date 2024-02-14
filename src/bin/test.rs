#![no_std]

use core::panic::PanicInfo;

#[]
fn main() {

}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    loop {}
}