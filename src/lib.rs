#![no_std]

use core::convert::{TryInto, TryFrom};

use embedded_hal as ehal;
use ehal::blocking::spi::{Write, Transfer};
use ehal::digital::v2::OutputPin;
use ehal::blocking::delay::DelayUs;
use ehal::spi::FullDuplex;

pub mod register;
pub mod command;
pub mod spi;

pub mod ads1292;
pub mod ads1298;

pub struct Ads1292Family;
pub struct Ads1298Family;

#[derive(Debug)]
pub enum Ads129xError<E> {
    /// Identification register read problem (probably unsupported device)
    IdRegRead(register::id::IdRegError),
    /// Read bytes is invalid register value
    ReadInterpret(u8),
    /// Spi transport error
    Spi(E),
}

macro_rules! impl_cmd {
    ($fn_name:ident, $command:ident) => {
        pub fn $fn_name(&mut self, delay: impl DelayUs<u32>) -> Ads129xResult<(),E> {
            self.spi.write(&[command::Command::$command as u8], delay)?;
            Ok(())
        }
    }
}

macro_rules! write_reg {
    (FAM: $family_path:ident, FN: $fn_name:ident, REG: $reg_name:ident ($param_path:ident::$param_ty:ident => $reg_path:ident::$reg_ty:ident)) => {
        pub fn $fn_name(&mut self, param: $family_path::$param_path::$param_ty, delay: impl DelayUs<u32>) -> Ads129xResult<(),E> {
            let mut words = [command::Command::WREG as u8 | $family_path::Register::$reg_name as u8, 0x00, $family_path::$reg_path::$reg_ty::from(param).0];
            let _ = self.spi.write(&mut words, delay)?;
            Ok(())
        }
    };
}

macro_rules! read_reg {
    (FAM: $family_path:ident, FN: $fn_name:ident, REG: $reg_name:ident ($param_path:ident::$param_ty:ident <= $reg_path:ident::$reg_ty:ident)) => {
        pub fn $fn_name(&mut self, delay: impl DelayUs<u32>) -> Ads129xResult<$family_path::$param_path::$param_ty,E>  {
            let mut words = [command::Command::RREG as u8 | $family_path::Register::$reg_name as u8, 0x00, 0xA5];
            let res = self.spi.transfer(&mut words, delay)?;

            let param = $family_path::$param_path::$param_ty::try_from($family_path::$reg_path::$reg_ty(res[2]))
                .map_err(|e|Ads129xError::ReadInterpret(e))?;

            Ok(param)
        }
    };
}


pub type Ads129xResult<T,E> = Result<T, Ads129xError<E>>;

pub struct Ads129x<SPI, NCS, DEV> {
    spi: spi::SpiDevice<SPI, NCS>,
    _d: core::marker::PhantomData<DEV>,
}

impl<SPI, NCS, DEV, E> Ads129x<SPI, NCS, DEV>
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

    pub fn read_id(&mut self, delay: impl DelayUs<u32>) -> Ads129xResult<register::id::DevModel,E> {
        let mut words = [command::Command::RREG as u8 | 0x00, 0x00, 0xA5];
        let res = self.spi.transfer(&mut words, delay)?;
            
        let model = register::id::DevModel::try_from(register::id::IdReg(res[2]))
            .map_err(|e|Ads129xError::IdRegRead(e))?;

        Ok(model)
    }

    pub fn destroy(self) -> (SPI, NCS) {
        self.spi.destroy()
    }
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1292Family> 
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    pub fn new_ads1292(spi: SPI, ncs: NCS) -> Self {
        Self {
            spi: spi::SpiDevice::new(spi, ncs),
            _d: core::marker::PhantomData,
        }
    }

    read_reg!(FAM: ads1292, FN: config, REG: CONFIG1 (conf::Config <= conf::Config1Reg));
    write_reg!(FAM: ads1292, FN: set_config, REG: CONFIG1 (conf::Config => conf::Config1Reg));
}

impl<SPI, NCS, E> Ads129x<SPI, NCS, Ads1298Family> 
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E> + FullDuplex<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    E: core::fmt::Debug,
{
    pub fn new_ads1298(spi: SPI, ncs: NCS) -> Self {
        Self {
            spi: spi::SpiDevice::new(spi, ncs),
            _d: core::marker::PhantomData,
        }
    }

    read_reg!(FAM: ads1298, FN: config, REG: CONFIG1 (conf::Config <= conf::Config1Reg));
    write_reg!(FAM: ads1298, FN: set_config, REG: CONFIG1 (conf::Config => conf::Config1Reg));
    read_reg!(FAM: ads1298, FN: test_signal_config, REG: CONFIG2 (conf::TestSignalConfig <= conf::Config2Reg));
    write_reg!(FAM: ads1298, FN: set_test_signal_config, REG: CONFIG2 (conf::TestSignalConfig => conf::Config2Reg));
    read_reg!(FAM: ads1298, FN: test_rld_config, REG: CONFIG3 (conf::RldConfig <= conf::Config3Reg));
    write_reg!(FAM: ads1298, FN: set_rld_config, REG: CONFIG3 (conf::RldConfig => conf::Config3Reg));
}

impl<E> From<E> for Ads129xError<E> {
    fn from(e: E) -> Self {
        Self::Spi(e)
    }
}
