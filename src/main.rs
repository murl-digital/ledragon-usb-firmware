#![no_std]
#![no_main]

mod usb_class;

use teensy4_panic as _;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP, SRC])]
mod app {
    use bsp::board;
    use teensy4_bsp::{
        self as bsp,
        hal::{
            pit::Pit2,
            usbd::{BusAdapter, EndpointMemory, EndpointState, Speed},
        },
    };

    use imxrt_log as logging;

    use super::usb_class::LEDragonUsbClass;
    use rtic_monotonics::systick::*;
    use usb_device::{
        bus::UsbBusAllocator,
        class::UsbClass,
        device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
        UsbError,
    };

    const PIT_DELAY_MS: u32 = board::PERCLK_FREQUENCY / 1_000 * 5;

    static EP_MEMORY: EndpointMemory<8192> = EndpointMemory::new();
    static EP_STATE: EndpointState = EndpointState::max_endpoints();

    #[local]
    struct Local {
        class: LEDragonUsbClass<'static, BusAdapter>,
        device: UsbDevice<'static, BusAdapter>,
        /// For driving the logging endpoint.
        //poller: logging::Poller,
        /// For periodically signaling activity.
        led: board::Led,
        pit: Pit2,
    }

    #[shared]
    struct Shared {}

    #[init(local = [bus: Option<UsbBusAllocator<BusAdapter>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let board::Resources {
            usb,
            pins,
            mut gpio2,
            pit: (_, _, mut pit, _),
            ..
        } = board::t40(ctx.device);
        let led = board::led(&mut gpio2, pins.p13);
        // pit.set_interrupt_enable(true);
        // pit.set_load_timer_value(PIT_DELAY_MS);
        // pit.enable();

        // Set up the logging system.
        //
        // There's various ways to control log levels at build- and run-time.
        // See the imxrt-log documentation for more information. This example
        // doesn't demonstrate any of that.
        //let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        let bus = BusAdapter::with_speed(usb, &EP_MEMORY, &EP_STATE, Speed::High);
        bus.set_interrupts(true);

        let bus = ctx.local.bus.insert(UsbBusAllocator::new(bus));
        let class = LEDragonUsbClass::new(bus);
        let device = UsbDeviceBuilder::new(bus, UsbVidPid(0x6942, 0x6942))
            .manufacturer("wick and drac's")
            .product("LEDragon")
            .max_packet_size_0(64)
            .build();

        // If the LED turns on, we've made it past init.
        led.set();

        (
            Shared {},
            Local {
                class,
                device,
                led,
                pit,
            },
        )
    }

    // /// This task periodically logs data.
    // ///
    // /// You won't see all the log levels until you configure your build. See the
    // /// top-level docs for more information.
    // #[task(local = [lmao: () = ()])]
    // async fn make_logs(cx: make_logs::Context) {
    //     // let make_logs::LocalResources { led, .. } = cx.local;

    //     // let mut counter = 0u32;
    //     // loop {
    //     //     led.toggle();
    //     //     Systick::delay(250.millis()).await;

    //     //     log::trace!("TRACE: {}", counter);

    //     //     if counter % 3 == 0 {
    //     //         log::debug!("DEBUG: {}", counter);
    //     //     }

    //     //     if counter % 5 == 0 {
    //     //         log::info!("INFO: {}", counter);
    //     //     }

    //     //     if counter % 7 == 0 {
    //     //         log::warn!("WARN: {}", counter);
    //     //     }

    //     //     if counter % 31 == 0 {
    //     //         log::error!("ERROR: {}", counter);
    //     //     }

    //     //     counter = counter.wrapping_add(1);
    //     // }
    // }
    //
    #[task()]
    async fn respond(ctx: respond::Context, packet: [u8; 1024]) {
        let mut thing = 1.1;
        for _ in 0..1000 {
            thing *= thing;
        }
    }

    /// This task runs when the USB1 interrupt activates.
    /// Simply poll the logger to control the logging process.
    #[task(binds = USB_OTG1, local = [led, class, device, configured: bool = false])]
    fn usb_interrupt(ctx: usb_interrupt::Context) {
        let usb_interrupt::LocalResources {
            device,
            class,
            configured,
            led,
            ..
        } = ctx.local;

        if device.poll(&mut [class]) {
            let mut data: [u8; 1024] = [0; 1024];
            //led.toggle();
            if device.state() == UsbDeviceState::Configured {
                if !*configured {
                    device.bus().configure();
                }
                *configured = true;

                class.poll();

                match class.read(&mut data) {
                    Ok(_) => {
                        led.toggle();
                        respond::spawn(data);
                        //class.write(b"data get!");
                    }
                    Err(UsbError::WouldBlock) => {
                        //class.write(b"would block");
                    }
                    Err(UsbError::BufferOverflow) => {
                        led.clear();
                        class.write(b"buffer overflow??");
                    }
                    Err(_) => {
                        led.set();
                    }
                }
            } else {
                *configured = false;
            }
        }
    }
}
