// Warning: This example is compiling for target `thumbv7m-none-eabi`
// but have never been tested for this target.
//
// Treat it as a reference only!

#![no_std]
#![no_main]

extern crate alloc;

use core::{
    alloc::{GlobalAlloc, Layout},
    panic::PanicInfo,
};

use pubnub::{
    core::{
        transport::{blocking::Transport, PUBNUB_DEFAULT_BASE_URL},
        transport_request::TransportRequest,
        transport_response::TransportResponse,
        PubNubError,
    },
    Keyset, PubNubClientBuilder,
};

// As getrandom crate has limited support of targets, we need to provide custom
// implementation of `getrandom` function.
fn custom_random(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // We're using `42` as a random number, because it's the answer
    // to the Ultimate Question of Life, the Universe, and Everything.
    // In your program, you should use proper random number generator that is supported by your target.
    for i in buf.iter_mut() {
        *i = 42;
    }

    Ok(())
}

// This function is used to register the custom implementation of `getrandom` function.
#[no_mangle]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    let buf = unsafe {
        // fill the buffer with zeros
        core::ptr::write_bytes(dest, 0, len);
        // create mutable byte slice
        core::slice::from_raw_parts_mut(dest, len)
    };
    custom_random(buf)
}

// Many targets have very specific requirements for networking, so it's hard to
// provide a generic implementation.
// Depending on the target, you will probably need to implement `Transport` trait.
struct MyTransport;

impl Transport for MyTransport {
    fn send(&self, _: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let _hostname = PUBNUB_DEFAULT_BASE_URL;

        // Send your request here

        Ok(TransportResponse::default())
    }
}

// As our target does not have `std` library, we need to provide custom
// implementation of `GlobalAlloc` trait.
//
// In your program, you should use proper allocator that is supported by your target.
// Here you have dummy implementation that does nothing.
#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

// As our target does not have `std` library, we need to provide custom
// implementation of `panic_handler`.
//
// In your program, you should use proper panic handler that is supported by your target.
// Here you have dummy implementation that does nothing.
#[panic_handler]
fn panicking(_: &PanicInfo) -> ! {
    loop {}
}

// As we're using `no_main` attribute, we need to define `main` function manually.
// For this example we're using `extern "C"` ABI to make it work.
#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> usize {
    publish_example().map(|_| 0).unwrap()
}

// In standard subscribe examples we use `println` macro to print the result of the operation
// and it shows the idea of the example. `no_std` does not support `println` macro,
// so we're using `do_a_thing` function instead.
fn do_a_thing<T>(_: &T) {}

// As `no_std` does not support `Error` trait, we use `PubNubError` instead.
// In your program, you should handle the error properly for your use case.
fn publish_example() -> Result<(), PubNubError> {
    // As `no_std` does not support `env::var`, you need to set the keys manually.
    let publish_key = "SDK_PUB_KEY";
    let subscribe_key = "SDK_SUB_KEY";

    let client = PubNubClientBuilder::with_blocking_transport(MyTransport)
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    // As `no_std` does not support `println` macro, we're using `do_a_thing` function instead
    // to show possible usage of the result.

    let where_user = client.where_now().user_id("user_id").execute_blocking()?;

    where_user.iter().for_each(|channel| {
        do_a_thing(&channel);
    });

    Ok(())
}
