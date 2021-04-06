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

/// Register map description
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

    /// Basic device configuration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Config {
        /// Device mode
        pub mode: Mode,
        /// Oscillator clock output
        pub osc_clock_output: bool,
        /// Daisy chain or multiple readback mode
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

    /// Device mode
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

    /// Sample rate in high-resolution mode
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

    /// Sample rate in low power mode
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
        /// Configuration Register 1
        pub struct Config1Reg(u8);
        impl Debug;

        /// Output data rate
        pub output_date_rate, set_output_date_rate : 2, 0;

        /// `CLK` connection
        ///
        /// This bit determines if the internal oscillator signal is connected to the `CLK` pin when
        /// the `CLKSEL` pin = 1
        ///
        ///   - 0 = Oscillator clock output disabled
        ///   - 1 = Oscillator clock output enabled
        ///
        pub clock_enable, set_clock_enable : 5;

        /// Daisy-chain or multiple readback mode
        ///
        /// This bit determine swhich mode is enabled.
        ///
        ///   - 0 = Daisy-chain mode
        ///   - 1 = Multiple readback mode
        ///
        pub daisy_disable, set_daisy_disable : 6;

        /// High-resolution or low-power mode
        ///
        /// This bit determines whether the device runs in low-power or high-resolution mode.
        ///
        ///   - 0 = LP mode
        ///   - 1 = HR mode
        ///
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

    /// Test signal configuration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TestSignalConfig {
        /// Test signal frequency
        pub frequency: TestSignalFreq,
        /// Test signal amplitude
        pub amplitude: TestSignalAmp,
        /// Test signal source
        pub source: TestSignalSource,
        /// WCT chopping scheme
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

    /// Test signal frequency settings
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalFreq {
        /// Pulsed at `fCLK` / 2**21
        PulsedAtFclk_div_2_21 = 0b00,
        /// Pulsed at `fCLK` / 2**20
        PulsedAtFclk_div_2_20 = 0b01,
        /// Not used
        NotUsed = 0b10,
        /// At dc
        AtDC = 0b11,
    }

    /// Test signal amplitude settings
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalAmp {
        /// 1 × –(`VREFP`– `VREFN`)/ 2400V
        Mode_x1 = 0b0,
        /// 2 × –(`VREFP– `VREFN`)/ 2400V
        Mode_x2 = 0b1,
    }
    impl_from_enum_to_bool!(TestSignalAmp);

    /// Test signal source
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalSource {
        /// Test signals are driven externally
        External = 0b0,
        /// Test signals are driven internally
        Internal = 0b1,
    }
    impl_from_enum_to_bool!(TestSignalSource);

    /// WCT chopping scheme
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum WctChoppingFreq {
        /// Chopping frequency varies, see datasheet.
        Variable = 0b0,
        /// Chopping frequency constant at `fMOD`/ 16
        Const = 0b1,
    }
    impl_from_enum_to_bool!(WctChoppingFreq);

    // 0x02
    bitfield! {
        /// Configuration register 2
        ///
        /// Configures the test signal generation
        pub struct Config2Reg(u8);
        impl Debug;

        /// Test signal frequency
        ///
        /// These bits determine the calibration signal frequency.
        ///   - 00 = Pulsed at `fCLK` / 2**21
        ///   - 01 = Pulsed at `fCLK` / 2**20
        ///   - 10 = Not used
        ///   - 11 = At dc
        ///
        pub test_freq, set_test_freq : 1, 0;

        /// Test signal amplitude
        ///
        /// These bits determine the calibration signal amplitude
        ///
        ///   - 0 = 1 × –(`VREFP`– `VREFN`)/ 2400V
        ///   - 1 = 2 × –(`VREFP`– `VREFN`)/ 2400V
        ///
        pub test_amp, set_test_amp : 2;

        /// TEST source
        ///
        /// This bit determines the source for the test signal.
        ///
        ///   - 0 = Test signals are driven externally
        ///   - 1 = Test signals are generated internally
        ///
        pub int_test, set_int_test : 4;

        /// WCT chopping scheme
        ///
        /// This bit determines whether the chopping frequency of WCT amplifiers is variable or
        /// fixed
        ///
        ///   - 0 = Chopping frequency varies, see datasheet.
        ///   - 1 = Choppingfrequencyconstantat `fMOD`/ 16
        ///
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

    /// Configures multireference and RLD operation
    #[allow(non_snake_case)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct RldConfig {
        /// RLD lead-off status
        ///
        ///   - false = connected
        ///   - true = not connected
        ///
        pub leadoff_status: bool,

        /// RLD sense function enable
        pub leadoff_sense_enable: bool,

        /// RLD buffer power enable
        pub buffer_power_enable: bool,

        /// `RLDREF` signal source
        pub ref_source: RldRefSource,

        /// RLD measurement
        ///   - 0 = Open
        ///   - 1 = `RLD_IN` signal is routed to the channel that has the MUX_Setting 010 (VREF)
        pub measurement_enable: bool,

        /// Reference voltage 4V enable
        pub vref_4V_enable: bool,

        /// Power-down reference buffer enable
        pub ref_buffer_enable: bool,
    }

    /// Determines the `RLDREF` signal source
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum RldRefSource {
        /// `RLDREF` signal fed externally
        External = 0b0,
        /// `RLDREF` signal (`AVDD`– `AVSS`)/ 2 generated internally
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
        /// Configuration register 3
        ///
        /// Configures multireference and RLD operation
        ///
        pub struct Config3Reg(u8);
        impl Debug;

        /// RLD lead-off status
        ///
        /// This bit determinesthe RLD status.
        ///
        /// **Readonly**
        ///
        ///   - 0 = RLD is connected
        ///   - 1 = RLD is not connected
        ///
        pub rld_stat, _ : 0;

        /// RLD sense function
        ///
        /// This bit enables the RLD sense function.
        ///
        ///   - 0 = RLD sense is disabled
        ///   - 1 = RLD sense is enabled
        ///
        pub rld_loff_sens, set_rld_loff_sens : 1;

        /// RLD buffer power
        ///
        /// This bit determines the RLD buffer power state
        ///
        ///   - 0 = RLD buffer is powereddown
        ///   - 1 = RLD buffer is enabled
        ///
        pub pd_rld, set_pd_rld : 2;

        /// `RLDREF` signal
        ///
        /// This bit determines the `RLDREF` signal source
        ///
        ///   - 0 = `RLDREF` signal fed externally
        ///   - 1 = `RLDREF` signal (`AVDD`– `AVSS`)/ 2 generated internally
        ///
        pub rldref_int, set_rldref_int : 3;

        /// RLD measurement
        ///
        /// This bit enables RLD measurement
        ///   - 0 = Open
        ///   - 1 = `RLD_IN` signal is routed to the channel that has the MUX_Setting 010 (VREF)
        ///
        pub rld_meas, set_rld_meas : 4;
        /// Reference voltage
        ///
        /// This bit determines the reference voltage, `VREFP`
        ///
        ///   - 0 = `VREFP` is set to 2.4 V
        ///   - 1 = `VREFP` is set to 4 V (use only with a 5-V analog supply)
        ///
        pub vref_4v, set_vref_4v : 5;

        /// Reserved
        ///
        /// Always 0x1
        ///
        _, set_reserved : 6;

        /// Power-down reference buffer
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

pub mod chan {
    use super::*;

    /// Individual channel settings
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Chan {
        PowerUp {
            input: ChannelInput,
            gain: ChannelGain,
        },
        PowerDown,
    }

    impl Default for Chan {
        fn default() -> Self {
            Chan::PowerUp {
                input: ChannelInput::Normal,
                gain: ChannelGain::X6,
            }
        }
    }

    /// Channel Input
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ChannelInput {
        /// Normal electrode input
        Normal = 0b000,
        /// Input shorted (for offset or noise measurements)
        Shorted = 0b001,
        /// Used in conjunction with `RLD_MEAS` bit for RLD measurements.
        Rld = 0b010,
        /// MVDD for supply measurement
        MVDD = 0b011,
        /// Temperature sensor
        Temp = 0b100,
        /// Test signal
        TestSig = 0b101,
        /// RLD_DRP (positiv eelectrode is the driver)
        RldDrp = 0b110,
        /// RLD_DRN (negative electrode is the driver)
        RldDrn = 0b111,
    }

    /// PGA gain
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ChannelGain {
        X6 = 0b000,
        X1 = 0b001,
        X2 = 0b010,
        X3 = 0b011,
        X4 = 0b100,
        X8 = 0b101,
        X12 = 0b110,
    }

    bitfield! {
        /// Individual channel settings
        ///
        /// The CH[1:8]SET control register configures the power mode, PGAgain, and multiplexer
        /// settings channels
        ///
        pub struct ChanSetReg(u8);
        impl Debug;

        /// Channel Input
        ///
        /// These bits determine the channel input selection.
        ///
        ///   - 000 = Normal electrode input
        ///   - 001 = Input shorted (for offset or noise measurements)
        ///   - 010 = Used in conjunction with `RLD_MEAS` bit for RLD measurements.
        ///   - 011 = MVDD for supply measurement
        ///   - 100 = Temperature sensor
        ///   - 101 = Test signal
        ///   - 110 = RLD_DRP (positiv eelectrode is the driver)
        ///   - 111 = RLD_DRN (negative electrode is the driver)
        ///
        pub mux, set_mux : 2,0;

        /// PGA gain
        ///
        /// These bits determine the PGA gain setting.
        ///   - 000 =  6
        ///   - 001 =  1
        ///   - 010 =  2
        ///   - 011 =  3
        ///   - 100 =  4
        ///   - 101 =  8
        ///   - 110 = 12
        ///
        pub gain, set_gain : 6, 4;

        /// Power-down
        ///
        /// This bit determines the channel power mode for the corresponding channel.
        ///
        ///   - 0 = Normaloperation
        ///   - 1 = Channelpower-down.
        ///
        /// When powering down a channel,TI recommends that the channel be set to
        /// input short by setting the appropriate MUXn[2:0]= 001 of the CHnSET register
        ///
        pub pd, set_pd: 7;
    }

    impl From<Chan> for ChanSetReg {
        fn from(chan: Chan) -> Self {
            let mut reg = ChanSetReg(0);
            match chan {
                Chan::PowerUp { input, gain } => {
                    reg.set_mux(input as u8);
                    reg.set_gain(gain as u8);
                    reg.set_pd(false);
                }
                Chan::PowerDown => {
                    reg.set_mux(ChannelInput::Shorted as u8);
                    reg.set_pd(true);
                }
            }
            reg
        }
    }

    impl TryFrom<ChanSetReg> for Chan {
        type Error = u8;

        fn try_from(reg: ChanSetReg) -> Result<Self, Self::Error> {
            Ok(if reg.pd() {
                Chan::PowerDown
            } else {
                Chan::PowerUp {
                    input: ChannelInput::try_from(reg.mux()).map_err(|_| reg.0)?,
                    gain: ChannelGain::try_from(reg.gain()).map_err(|_| reg.0)?,
                }
            })
        }
    }
}
