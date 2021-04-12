use ehal::blocking::delay::DelayUs;
use ehal::blocking::spi::{Transfer, Write};
use ehal::digital::v2::OutputPin;
use ehal::spi::FullDuplex;
use embedded_hal as ehal;

/// A SPI device also triggering the nCS-pin when suited.
pub struct SpiDevice<SPI, NCS> {
    /// Underlying peripheral
    spi: SPI,
    /// nCS
    ncs: NCS,
}

impl<SPI, NCS, E> SpiDevice<SPI, NCS>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
{
    /// Create a new SPI device
    pub fn new(spi: SPI, mut ncs: NCS) -> Self {
        let _ = ncs.set_high();

        SpiDevice { spi, ncs }
    }

    /// Transfer the buffer to the device, the passed buffer will contain the
    /// read data.
    #[inline]
    pub fn transfer<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
        mut delay: impl DelayUs<u32>,
    ) -> Result<&'buf [u8], E> {
        let _ = self.ncs.set_low();
        delay.delay_us(40);

        let res = self.spi.transfer(buffer);

        delay.delay_us(40);
        let _ = self.ncs.set_high();
        delay.delay_us(20);
        // Drop out of function with SPIError only after setting NCS.
        Ok(res?)
    }

    /// Write a number of bytes to the device.
    #[inline]
    pub fn write(&mut self, buffer: &[u8], mut delay: impl DelayUs<u32>) -> Result<(), E> {
        let _ = self.ncs.set_low();
        delay.delay_us(40);

        let res = self.spi.write(buffer);

        delay.delay_us(40);
        let _ = self.ncs.set_high();
        delay.delay_us(20);

        res?; // Drop out of function with SPIError only after setting NCS.
        Ok(())
    }

    pub fn destroy(self) -> (SPI, NCS) {
        (self.spi, self.ncs)
    }
}
