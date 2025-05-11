use usb_device::{
    bus::{InterfaceNumber, UsbBus, UsbBusAllocator},
    class::UsbClass,
    endpoint::{EndpointIn, EndpointOut},
};

pub struct LEDragonUsbClass<'ep, B: UsbBus> {
    interface: InterfaceNumber,
    write_endpoint: EndpointIn<'ep, B>,
    read_endpoint: EndpointOut<'ep, B>,
}

impl<'ep, B: UsbBus> LEDragonUsbClass<'ep, B> {
    pub fn new<'alloc: 'ep>(alloc: &'alloc UsbBusAllocator<B>) -> Self {
        Self {
            interface: alloc.interface(),
            write_endpoint: alloc.interrupt(64, 255),
            read_endpoint: alloc.bulk(64),
        }
    }

    pub fn read(&self, data: &mut [u8]) -> usb_device::Result<usize> {
        self.read_endpoint.read(data)
    }

    pub fn write(&self, data: &[u8]) -> usb_device::Result<usize> {
        self.write_endpoint.write(data)
    }
}

impl<'ep, B: UsbBus> UsbClass<B> for LEDragonUsbClass<'ep, B> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut usb_device::descriptor::DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.interface(self.interface, 0xff, 0, 0)?;
        writer.endpoint(&self.write_endpoint)?;
        writer.endpoint(&self.read_endpoint)?;

        Ok(())
    }
}
