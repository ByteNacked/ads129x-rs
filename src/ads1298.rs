#![allow(non_camel_case_types)]

use core::convert::TryFrom;

use bitfield::bitfield;
use num_enum::TryFromPrimitive;

macro_rules! impl_from_enum_to_bool {
    ($enum_name:ident) => {
        impl From<$enum_name> for bool {
            fn from(v: $enum_name) -> bool {
                if v as u8 == 0x00 {
                    false
                } else {
                    true
                }
            }
        }
    };
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    /// ID Control Register (Factory-Programmed, Read-Only)
    ID = 0x00,
    /// Configuration Register 1
    CONFIG1 = 0x01,
    /// Configuration Register 2
    CONFIG2 = 0x02,
    /// Configuration Register 3
    CONFIG3 = 0x03,
    /// Lead-Off Control Register
    LOFF = 0x04,

    /// Channel 1 Settings
    CH1SET = 0x05,
    /// Channel 2 Settings
    CH2SET = 0x06,
    /// Channel 3 Settings
    CH3SET = 0x07,
    /// Channel 4 Settings
    CH4SET = 0x08,
    /// Channel 5 Settings
    CH5SET = 0x09,
    /// Channel 6 Settings
    CH6SET = 0x0A,
    /// Channel 7 Settings
    CH7SET = 0x0B,
    /// Channel 8 Settings
    CH8SET = 0x0C,

    /// Right Leg Drive Positive Sense Selection
    RLD_SENSP = 0x0D,
    /// Right Leg Drive Negative Sense Selection
    RLD_SENSN = 0x0E,
    /// Lead-Off Positive Sense Selection
    LOFF_SENSP = 0x0F,
    /// Lead-Off Negative Sense Selection
    LOFF_SENSN = 0x10,
    /// Lead-off Flip
    LOFF_FLIP = 0x11,
    /// Lead-Off Positive Signal Status
    LOFF_STATP = 0x12,
    /// Lead-Off Negative Signal Status
    LOFF_STATN = 0x13,

    /// General-Purpose I/O Register
    GPIO = 0x14,
    /// Pace Detect Register
    PACE = 0x15,
    /// Respiration Control Register
    RESP = 0x16,
    /// Configuration Register 4
    CONFIG4 = 0x17,
    /// Wilson Central Terminal and Augmented Lead Control Register 1
    WCT1 = 0x18,
    /// Wilson Central Terminal and Augmented Lead Control Register 2
    WCT2 = 0x19,
}

pub mod conf {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Config {
        pub mode: Mode,
        pub osc_clock_output: bool,
        pub daisy_chain: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            Config {
                mode: Mode::default(),
                osc_clock_output: false,
                daisy_chain: true,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Mode {
        HighResolution(SampleRateHR),
        LowPower(SampleRateLP),
    }

    impl Default for Mode {
        fn default() -> Self {
            Mode::LowPower(SampleRateLP::Sps250)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum SampleRateHR {
        KSps32 = 0b000,
        KSps16 = 0b001,
        Sps8k = 0b010,
        Sps4k = 0b011,
        Sps2k = 0b100,
        Sps1k = 0b101,
        Sps500 = 0b110,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum SampleRateLP {
        KSps16 = 0b000,
        KSps8 = 0b001,
        KSps4 = 0b010,
        KSps2 = 0b011,
        KSps1 = 0b100,
        Sps500 = 0b101,
        Sps250 = 0b110,
    }

    // 0x01
    bitfield! {
        pub struct Config1Reg(u8);
        impl Debug;
        pub output_date_rate, set_output_date_rate : 2, 0;
        pub clock_enable, set_clock_enable : 5;
        pub daisy_disable, set_daisy_disable : 6;
        pub high_resolution, set_high_resolution : 7;
    }

    impl From<Config> for Config1Reg {
        fn from(config: Config) -> Self {
            let (high_resolution, output_date_rate) = match config.mode {
                Mode::HighResolution(data_rate) => (true, data_rate as u8),
                Mode::LowPower(data_rate) => (false, data_rate as u8),
            };

            let mut reg = Config1Reg(0);
            reg.set_output_date_rate(output_date_rate);
            reg.set_clock_enable(config.osc_clock_output);
            reg.set_daisy_disable(!config.daisy_chain);
            reg.set_high_resolution(high_resolution);
            reg
        }
    }

    impl TryFrom<Config1Reg> for Config {
        type Error = u8;

        fn try_from(reg: Config1Reg) -> Result<Self, Self::Error> {
            Ok(Config {
                mode: match reg.high_resolution() {
                    true => Mode::HighResolution(
                        SampleRateHR::try_from(reg.output_date_rate()).map_err(|_| reg.0)?,
                    ),
                    false => Mode::LowPower(
                        SampleRateLP::try_from(reg.output_date_rate()).map_err(|_| reg.0)?,
                    ),
                },
                osc_clock_output: reg.clock_enable(),
                daisy_chain: !reg.daisy_disable(),
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TestSignalConfig {
        pub frequency: TestSignalFreq,
        pub amplitude: TestSignalAmp,
        pub source: TestSignalSource,
        pub wct_chop: WctChoppingFreq,
    }

    impl Default for TestSignalConfig {
        fn default() -> Self {
            TestSignalConfig {
                frequency: TestSignalFreq::PulsedAtFclk_div_2_21,
                amplitude: TestSignalAmp::Mode_x1,
                source: TestSignalSource::External,
                wct_chop: WctChoppingFreq::Variable,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalFreq {
        PulsedAtFclk_div_2_21 = 0b00,
        PulsedAtFclk_div_2_20 = 0b01,
        NotUsed = 0b10,
        AtDC = 0b11,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalAmp {
        /// 1 × –(VREFP– VREFN)/ 2400V
        Mode_x1 = 0b0,
        /// 2 × –(VREFP– VREFN)/ 2400V
        Mode_x2 = 0b1,
    }
    impl_from_enum_to_bool!(TestSignalAmp);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalSource {
        External = 0b0,
        Internal = 0b1,
    }
    impl_from_enum_to_bool!(TestSignalSource);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum WctChoppingFreq {
        /// Chopping frequency varies
        Variable = 0b0,
        /// Chopping frequency constant at fMOD/ 16
        Const = 0b1,
    }
    impl_from_enum_to_bool!(WctChoppingFreq);

    // 0x02
    bitfield! {
        /// Configuration register 2
        /// configures the test signal generation
        pub struct Config2Reg(u8);
        impl Debug;
        /// Test signal frequency
        /// These bits determine the calibration signal frequency.
        pub test_freq, set_test_freq : 1, 0;
        /// Test signal amplitude
        /// These bits determine the calibration signal amplitude
        pub test_amp, set_test_amp : 2;
        /// TEST source
        /// This bit determines the source for the test signal.
        pub int_test, set_int_test : 4;
        /// WCT chopping scheme
        /// This bit determines whether the chopping frequency of WCT amplifiers is variable or
        /// fixed
        pub wct_chop, set_wct_chop : 5;
    }

    impl From<TestSignalConfig> for Config2Reg {
        fn from(config: TestSignalConfig) -> Config2Reg {
            let mut reg = Config2Reg(0);
            reg.set_test_freq(config.frequency as u8);
            reg.set_test_amp(config.amplitude.into());
            reg.set_int_test(config.source.into());
            reg.set_wct_chop(config.wct_chop.into());
            reg
        }
    }

    impl TryFrom<Config2Reg> for TestSignalConfig {
        type Error = u8;

        fn try_from(reg: Config2Reg) -> Result<Self, Self::Error> {
            Ok(TestSignalConfig {
                frequency: TestSignalFreq::try_from(reg.test_freq() as u8).map_err(|_| reg.0)?,
                amplitude: TestSignalAmp::try_from(reg.test_amp() as u8).map_err(|_| reg.0)?,
                source: TestSignalSource::try_from(reg.int_test() as u8).map_err(|_| reg.0)?,
                wct_chop: WctChoppingFreq::try_from(reg.wct_chop() as u8).map_err(|_| reg.0)?,
            })
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct RldConfig {
        /// RLD lead-off status
        ///
        /// (WARNING: inverse logic)
        ///
        ///   - false = connected
        ///   - true = not connected
        ///
        pub leadoff_status: bool,

        /// RLD sense function
        pub leadoff_sense_enable: bool,

        /// RLD buffer power
        pub buffer_power_enable: bool,

        /// RLDREF signal source
        pub ref_source: RldRefSource,

        /// RLD measurement
        ///   - 0 = Open
        ///   - 1 = RLD_IN signal is routed to the channel that has the MUX_Setting 010 (VREF)
        pub measurement_enable: bool,

        /// Reference voltage 4V enable
        pub vref_4V_enable: bool,

        /// Power-down reference buffer
        pub ref_buffer_enable: bool,
    }

    /// Determines the RLDREF signal source
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum RldRefSource {
        /// RLDREF signal fed externally
        External = 0b0,
        /// RLDREF signal (AVDD– AVSS)/ 2 generated internally
        Interanl = 0b1,
    }
    impl_from_enum_to_bool!(RldRefSource);

    impl Default for RldRefSource {
        fn default() -> Self {
            RldRefSource::External
        }
    }

    // 0x03
    bitfield! {
        /// # Configuration register 3
        /// configures multireference and RLD operation
        pub struct Config3Reg(u8);
        impl Debug;

        /// ## RLD lead-off status
        ///
        /// This bit determinesthe RLD status.
        ///
        /// **Readonly**
        ///
        ///   - 0 = RLDis connected
        ///   - 1 = RLDis not connected
        ///
        pub rld_stat, set_rld_stat : 0;

        /// ## RLD sense function
        ///
        /// This bit enables the RLD sense function.
        ///
        ///   - 0 = RLDsenseis disabled
        ///   - 1 = RLDsenseis enabled
        ///
        pub rld_loff_sens, set_rld_loff_sens : 1;

        /// ## RLD buffer power
        ///
        /// This bit determines the RLD buffer power state
        ///
        ///   - 0 = RLDbufferis powereddown
        ///   - 1 = RLDbufferis enabled
        ///
        pub pd_rld, set_pd_rld : 2;

        /// ## RLDREF signal
        ///
        /// This bit determines the RLDREF signal source
        ///
        ///   - 0 = RLDREFsignalfed externally
        ///   - 1 = RLDREFsignal(AVDD– AVSS)/ 2 generatedinternally
        ///
        pub rldref_int, set_rldref_int : 3;

        /// ## RLD measurement
        ///
        /// This bit enables RLD measurement
        ///   - 0 = Open
        ///   - 1 = RLD_IN signal is routed to the channel that has the MUX_Setting 010 (VREF)
        ///
        pub rld_meas, set_rld_meas : 4;
        /// ## Reference voltage
        ///
        /// This bit determines the reference voltage, VREFP
        ///
        ///   - 0 = VREFP is set to 2.4 V
        ///   - 1 = VREFP is set to 4 V (use only with a 5-V analog supply)
        ///
        pub vref_4v, set_vref_4v : 5;

        /// ## Reserved
        ///
        /// Always 0x1
        ///
        _, set_reserved : 6;

        /// ## Power-down reference buffer
        ///
        /// This bit determines the power-down reference buffer state
        ///
        ///   - 0 = Power-down internal reference buffer
        ///   - 1 = Enable internal reference buffer
        ///
        pub pd_refbuf, set_pd_refbuf : 7;
    }

    impl From<RldConfig> for Config3Reg {
        fn from(conf: RldConfig) -> Self {
            let mut reg = Config3Reg(0);
            reg.set_rld_loff_sens(conf.leadoff_sense_enable);
            reg.set_pd_rld(conf.buffer_power_enable);
            reg.set_rldref_int(conf.ref_source.into());
            reg.set_rld_meas(conf.measurement_enable);
            reg.set_vref_4v(conf.vref_4V_enable);
            reg.set_reserved(true);
            reg.set_pd_refbuf(conf.ref_buffer_enable);
            reg
        }
    }

    impl TryFrom<Config3Reg> for RldConfig {
        type Error = u8;

        fn try_from(reg: Config3Reg) -> Result<Self, Self::Error> {
            Ok(RldConfig {
                leadoff_status: reg.rld_stat(),
                leadoff_sense_enable: reg.rld_loff_sens(),
                buffer_power_enable: reg.pd_rld(),
                ref_source: RldRefSource::try_from(reg.rldref_int() as u8).map_err(|_| reg.0)?,
                measurement_enable: reg.rld_meas(),
                vref_4V_enable: reg.vref_4v(),
                ref_buffer_enable: reg.pd_refbuf(),
            })
        }
    }
}

pub mod chan {}
