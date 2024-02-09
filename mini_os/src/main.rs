#![no_std]
#![no_main]
mod vga_buffer;
use core::panic::PanicInfo;
// static HELLO: &[u8] = b"Hello World!";
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // vga_buffer::write_something();
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello world").unwrap();
    // write!(vga_buffer::WRITER.lock(), "some numbers: {} {}", 10, 3.14).unwrap();
    println!("hello world{}", "!");
    panic!("Some panic message!");
    // loop {}
}
