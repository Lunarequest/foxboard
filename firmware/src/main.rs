//! The starter code slowly blinks the LED and sets up
//! USB logging. It periodically logs messages over USB.
//!
//! Despite targeting the Teensy 4.0, this starter code
//! should also work on the Teensy 4.1 and Teensy MicroMod.
//! You should eventually target your board! See inline notes.
//!
//! This template uses [RTIC v2](https://rtic.rs/2/book/en/)
//! for structuring the application.

#![no_std]
#![no_main]

extern crate alloc;
mod usb;

use embedded_alloc::Heap;
use teensy4_panic as _;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use teensy4_bsp::{self as bsp};

    use imxrt_log as logging;

    // If you're using a Teensy 4.1 or MicroMod, you should eventually
    // change 't40' to 't41' or micromod, respectively.
    use board::t41 as my_board;

    use crate::usb::USBEvent;
    use alloc::collections::vec_deque::VecDeque;
    use rtic_monotonics::systick::{Systick, *};
    use teensy4_bsp::{
        hal::{gpio, iomuxc},
        pins,
    };

    type Input = gpio::Input<pins::t41::P7>;

    const PIN_CONFIG: iomuxc::Config =
        iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown100k));

    #[shared]
    struct Shared {
        event: VecDeque<USBEvent>,
    }

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        input: Input,
        /// The LED on pin 13.
        led: board::Led,
        /// A poller to control USB logging.
        poller: logging::Poller,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio2,
            mut pins,
            usb,
            ..
        } = my_board(cx.device);

        let led = board::led(&mut gpio2, pins.p13);

        iomuxc::configure(&mut pins.p7, PIN_CONFIG);
        let input = gpio2.input(pins.p7);

        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );
        log::info!("init complete");

        blink::spawn().unwrap();
        (
            Shared {
                event: VecDeque::new(),
            },
            Local { led, poller, input },
        )
    }

    #[task(local = [led, input,])]
    async fn blink(cx: blink::Context) {
        log::info!("Hello from your Teensy 41!");
        let blink::LocalResources { led, input, .. } = cx.local;
        loop {
            led.set();
            let status = input.is_set();
            log::info!("the input status is {status}");
            led.clear();
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
