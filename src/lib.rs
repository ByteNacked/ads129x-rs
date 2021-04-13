#![no_std]

use core::convert::TryFrom;

use ehal::blocking::delay::DelayUs;
use ehal::blocking::spi::{Transfer, Write};
use ehal::digital::v2::OutputPin;
use ehal::spi::FullDuplex;
use embedded_hal as ehal;

#[macro_use]
mod util;
pub mod command;
pub mod common;
pub mod data;
pub mod spi;

pub mod ads1292;
pub mod ads1298;

#[doc(hidden)]
pub struct Ads1292Family;
#[doc(hidden)]
pub struct Ads1298Family;

#[derive(Debug)]
pub enum Ads129xError<E> {
    /// Identification register read problem (probably unsupported device)
    IdRegRead(common::id::IdRegError),
    /// Read bytes is invalid register value
    ReadInterpret(u8),
    /// Status word missmatch
    StatusWordMissmatch(u8),
    /// Spi transport error
    Spi(E),
}

pub type Ads129xResult<T, E> = Result<T, Ads129xError<E>>;

pub struct Ads129x<SPI, NCS, DEV, const CH: usize> {
    spi: spi::SpiDevice<SPI, NCS>,
    _d:  core::marker::PhantomData<DEV>,
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1292Family, 2>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    /// Create ADS1292/ADS1292R device instance
    pub fn new_ads1292(spi: SPI, ncs: NCS) -> Self {
        Self {
            spi: spi::SpiDevice::new(spi, ncs),
            _d:  core::marker::PhantomData,
        }
    }
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1298Family, 4>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    /// Create ADS1294/ADS1294R device instance
    pub fn new_ads1294(spi: SPI, ncs: NCS) -> Self {
        Self {
            spi: spi::SpiDevice::new(spi, ncs),
            _d:  core::marker::PhantomData,
        }
    }
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1298Family, 6>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    /// Create ADS1296/ADS1296R device instance
    pub fn new_ads1296(spi: SPI, ncs: NCS) -> Self {
        Self {
            spi: spi::SpiDevice::new(spi, ncs),
            _d:  core::marker::PhantomData,
        }
    }
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1298Family, 8>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    /// Create ADS1298/ADS1298R device instance
    pub fn new_ads1298(spi: SPI, ncs: NCS) -> Self {
        Self {
            spi: spi::SpiDevice::new(spi, ncs),
            _d:  core::marker::PhantomData,
        }
    }
}

impl<SPI, NCS, DEV, E, const CH: usize> Ads129x<SPI, NCS, DEV, CH>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    impl_cmd!(wakeup_device, WAKEUP);
    impl_cmd!(set_standby_mode, STANDBY);
    impl_cmd!(reset_device, RESET);
    impl_cmd!(start_conv, START);
    impl_cmd!(stop_conv, STOP);
    impl_cmd!(set_continuous_mode, RDATAC);
    impl_cmd!(set_command_mode, SDATAC);

    pub fn read_id(&mut self, delay: impl DelayUs<u32>) -> Ads129xResult<common::id::DevModel, E> {
        let mut words = [command::Command::RREG as u8 | 0x00, 0x00, 0xA5];
        let res = self.spi.transfer(&mut words, delay)?;

        let model = common::id::DevModel::try_from(common::id::IdReg(res[2]))
            .map_err(|e| Ads129xError::IdRegRead(e))?;

        Ok(model)
    }

    pub fn destroy(self) -> (SPI, NCS) {
        self.spi.destroy()
    }
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1292Family, 2>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    read_reg!(FAM: ads1292, FN: config, REG: CONFIG1 (conf::Config <= conf::Config1Reg));
    write_reg!(FAM: ads1292, FN: set_config, REG: CONFIG1 (conf::Config => conf::Config1Reg));
}

impl<SPI, NCS, E, const CH: usize> Ads129x<SPI, NCS, Ads1298Family, CH>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    // Read data samples from ADC
    // Data samples are sign extend
    pub fn read_data(
        &mut self,
        data_frame: &mut data::DataFrame<CH>,
        mut delay: impl DelayUs<u32>,
    ) -> Ads129xResult<(), E> {
        // Read status_word/data
        {
            let _ = self.spi.ncs.set_low();
            delay.delay_us(40);

            // Read status word
            for idx in 0..data_frame.status_word.len() {
                nb::block!(self.spi.spi.send(0x00))?;
                data_frame.status_word[idx] = nb::block!(self.spi.spi.read())?;
            }
            // Read channels data, i24 big endian byte order
            for idx in 0..CH {
                let mut bb = [0x00u8; 4];
                nb::block!(self.spi.spi.send(0x00))?;
                bb[2] = nb::block!(self.spi.spi.read())?;
                nb::block!(self.spi.spi.send(0x00))?;
                bb[1] = nb::block!(self.spi.spi.read())?;
                nb::block!(self.spi.spi.send(0x00))?;
                bb[0] = nb::block!(self.spi.spi.read())?;
                // Assemble sample as le
                data_frame.data[idx] = i32::from_le_bytes(bb);
                // Sign extend i24 -> i32
                // On ARM should be optimized to SBFX instruction
                data_frame.data[idx] = data_frame.data[idx] << 8 >> 8;
            }

            delay.delay_us(40);
            let _ = self.spi.ncs.set_high();
            delay.delay_us(20);
        }

        // Validate status word
        let status_word = data_frame.status_word();
        if status_word.sync() != 0b1100 {
            return Err(Ads129xError::StatusWordMissmatch(status_word.sync()));
        }

        Ok(())
    }

    read_reg!(FAM: ads1298, FN: config, REG: CONFIG1 (conf::Config <= conf::Config1Reg));
    write_reg!(FAM: ads1298, FN: set_config, REG: CONFIG1 (conf::Config => conf::Config1Reg));
    read_reg!(FAM: ads1298, FN: test_signal_config, REG: CONFIG2 (conf::TestSignalConfig <= conf::Config2Reg));
    write_reg!(FAM: ads1298, FN: set_test_signal_config, REG: CONFIG2 (conf::TestSignalConfig => conf::Config2Reg));
    read_reg!(FAM: ads1298, FN: test_rld_config, REG: CONFIG3 (conf::RldConfig <= conf::Config3Reg));
    write_reg!(FAM: ads1298, FN: set_rld_config, REG: CONFIG3 (conf::RldConfig => conf::Config3Reg));

    read_reg!(FAM: ads1298, FN: leadoff_control, REG: LOFF (loff::LeadOffControl <= loff::LeadOffControlReg));
    write_reg!(FAM: ads1298, FN: set_leadoff_control, REG: LOFF (loff::LeadOffControl => loff::LeadOffControlReg));

    read_reg!(FAM: ads1298, FN: chan_1, REG: CH1SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_2, REG: CH2SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_3, REG: CH3SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_4, REG: CH4SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_5, REG: CH5SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_6, REG: CH6SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_7, REG: CH7SET (chan::Chan <= chan::ChanSetReg));
    read_reg!(FAM: ads1298, FN: chan_8, REG: CH8SET (chan::Chan <= chan::ChanSetReg));

    write_reg!(FAM: ads1298, FN: set_chan_1, REG: CH1SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_2, REG: CH2SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_3, REG: CH3SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_4, REG: CH4SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_5, REG: CH5SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_6, REG: CH6SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_7, REG: CH7SET (chan::Chan => chan::ChanSetReg));
    write_reg!(FAM: ads1298, FN: set_chan_8, REG: CH8SET (chan::Chan => chan::ChanSetReg));

    read_reg!(FAM: ads1298, FN: leadoff_sense_positive, REG: LOFF_SENSP (loff::LeadOffSense <= loff::LeadOffSenseReg));
    write_reg!(FAM: ads1298, FN: set_leadoff_sense_positive, REG: LOFF_SENSP (loff::LeadOffSense => loff::LeadOffSenseReg));
    read_reg!(FAM: ads1298, FN: leadoff_sense_negative, REG: LOFF_SENSN (loff::LeadOffSense <= loff::LeadOffSenseReg));
    write_reg!(FAM: ads1298, FN: set_leadoff_sense_negative, REG: LOFF_SENSN (loff::LeadOffSense => loff::LeadOffSenseReg));
    read_reg!(FAM: ads1298, FN: leadoff_flip, REG: LOFF_FLIP (loff::LeadOffFlip <= loff::LeadOffFlipReg));
    write_reg!(FAM: ads1298, FN: set_leadoff_flip, REG: LOFF_FLIP (loff::LeadOffFlip => loff::LeadOffFlipReg));

    read_reg!(FAM: ads1298, FN: gpio, REG: GPIO (gpio::Gpio <= gpio::GpioReg));
    write_reg!(FAM: ads1298, FN: set_gpio, REG: GPIO (gpio::Gpio => gpio::GpioReg));

    read_reg!(FAM: ads1298, FN: misc_config, REG: CONFIG4 (conf::MiscConfig <= conf::Config4Reg));
    write_reg!(FAM: ads1298, FN: set_misc_config, REG: CONFIG4 (conf::MiscConfig => conf::Config4Reg));
}

impl<E> From<E> for Ads129xError<E> {
    fn from(e: E) -> Self {
        Self::Spi(e)
    }
}
