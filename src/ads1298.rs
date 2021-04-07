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
        /// This bit determines the RLD status.
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

    /// Various configurations
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MiscConfig {
        pub leadoff_comparator_enable: bool,
        pub wct_to_rld_enable: bool,
        pub single_shot_mode: bool,
        pub respiration_freq: ResperationFreq,
    }

    impl Default for MiscConfig {
        fn default() -> Self {
            MiscConfig {
                leadoff_comparator_enable: false,
                wct_to_rld_enable: false,
                single_shot_mode: false,
                respiration_freq: ResperationFreq::KHz64,
            }
        }
    }

    /// Respiration modulation frequency
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ResperationFreq {
        /// 64 kHz modulation clock
        KHz64 = 0b000,
        /// 32 kHz modulationclock
        KHz32 = 0b001,
        /// 16kHz square wave on GPIO3 and GPIO04.
        /// Output on GPIO4 is 180 degree out of phase with GPIO3.
        KHz16 = 0b010,
        /// 8kHz square wave on GPIO3 and GPIO04.
        /// Output on GPIO4 is 180 degree out of phase with GPIO3.
        KHz8 = 0b011,
        /// 4kHz square wave on GPIO3 and GPIO04.
        /// Output on GPIO4 is 180 degree out of phase with GPIO3.
        KHz4 = 0b100,
        /// 2kHz square wave on GPIO3 and GPIO04.
        /// Output on GPIO4 is 180 degree out of phase with GPIO3.
        KHz2 = 0b101,
        /// 1kHz square wave on GPIO3 and GPIO04.
        /// Output on GPIO4 is 180 degree out of phase with GPIO3.
        KHz1 = 0b110,
        /// 500Hz square wave on GPIO3 and GPIO04.
        /// Output on GPIO4 is 180 degree out of phase with GPIO3.
        Hz500 = 0b111,
    }

    // 0x17
    bitfield! {
        /// Configuration Register 4
        pub struct Config4Reg(u8);
        impl Debug;

        /// Lead-off comparator power-down
        ///
        /// This bit powers down the lead-off comparators.
        ///
        ///   - 0 = Lead-off comparators disabled
        ///   - 1 = Lead-off comparators enabled
        ///
        pub pd_loff_comp, set_pd_loff_comp : 1;

        /// Connects the WCT to the RLD
        ///
        /// This bit connects WCT to RLD.
        ///
        ///   - 0 = WCTto RLD connection off
        ///   - 1 = WCTto RLD connection on
        ///
        pub wct_to_rld, set_wct_to_rld : 2;

        /// Single-shot conversion
        ///
        /// This bit sets the conversion mode.
        ///
        ///   - 0 = Continuous conversion mode
        ///   - 1 = Single-shotmode
        ///
        pub single_shot, set_single_shot : 3;

        /// Respiration modulation frequency
        ///
        /// These bits control the respiration control frequency when `RESP_CTRL`[1:0] = 10 or
        /// `RESP_CTRL`[1:0]= 10
        ///
        ///   - 000 = 64 kHz modulation clock
        ///   - 001 = 32 kHz modulationclock
        ///   - 010 = 16kHz square wave on GPIO3 and GPIO04. Output on GPIO4 is 180 degree out of phase with GPIO3.
        ///   - 011 = 8kHz square wave on GPIO3 and GPIO04. Output on GPIO4 is 180 degree out of phase with GPIO3.
        ///   - 100 = 4kHz square wave on GPIO3 and GPIO04. Output on GPIO4 is 180 degree out of phase with GPIO3.
        ///   - 101 = 2kHz square wave on GPIO3 and GPIO04. Output on GPIO4 is 180 degree out of phase with GPIO3.
        ///   - 110 = 1kHz square wave on GPIO3 and GPIO04. Output on GPIO4 is 180 degree out of phase with GPIO3.
        ///   - 111 = 500Hz square wave on GPIO3 and GPIO04. Output on GPIO4 is 180 degree out of phase with GPIO3.
        ///
        /// Modes 000 and 001 are modulation frequencies in internal and external respiration
        /// modes. In internal respiration mode, the control signals appear at the `RESP_MODP` and
        /// `RESP_MODN` terminals. All other bit settings generatei square waves as described above
        /// on GPIO4 and GPIO3.
        ///
        pub resp_freq, set_resp_freq : 7, 5;
    }

    impl From<MiscConfig> for Config4Reg {
        fn from(param: MiscConfig) -> Self {
            let mut reg = Config4Reg(0);
            reg.set_pd_loff_comp(param.leadoff_comparator_enable);
            reg.set_wct_to_rld(param.wct_to_rld_enable);
            reg.set_single_shot(param.single_shot_mode);
            reg.set_resp_freq(param.respiration_freq as u8);
            reg
        }
    }

    impl TryFrom<Config4Reg> for MiscConfig {
        type Error = u8;

        fn try_from(reg: Config4Reg) -> Result<Self, Self::Error> {
            Ok(MiscConfig {
                leadoff_comparator_enable: reg.pd_loff_comp(),
                wct_to_rld_enable: reg.wct_to_rld(),
                single_shot_mode: reg.single_shot(),
                respiration_freq: ResperationFreq::try_from(reg.resp_freq()).map_err(|_| reg.0)?,
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

pub mod loff {
    use super::*;

    /// Lead-off control configuration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LeadOffControl {
        pub frequency: LeadOffFreq,
        pub magnitude: LeadOffMagnitude,
        pub detection_mode: LeadOffDetectMode,
        pub comparator_threshold: LeadOffCompThreshold,
    }

    impl Default for LeadOffControl {
        fn default() -> Self {
            LeadOffControl {
                frequency: LeadOffFreq::Default,
                magnitude: LeadOffMagnitude::nA_6,
                detection_mode: LeadOffDetectMode::CurrentSource,
                comparator_threshold: LeadOffCompThreshold::PositiveSide(
                    CompPositiveSide::Pct_95_5,
                ),
            }
        }
    }

    /// Lead-off frequency
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum LeadOffFreq {
        /// Default value
        Default = 0b00,
        /// AC lead-offdetection at `fDR`/ 4
        AC = 0b01,
        /// Do not use
        NotUse = 0b10,
        /// DC lead-off detection turned on
        DC = 0b11,
    }

    /// Lead-off current magnitude
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum LeadOffMagnitude {
        nA_6 = 0b00,
        nA_12 = 0b01,
        nA_18 = 0b10,
        nA_24 = 0b11,
    }

    /// Lead-off detection mode
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    pub enum LeadOffDetectMode {
        CurrentSource = 0b0,
        PullUpDown = 0b1,
    }
    impl_from_enum_to_bool!(LeadOffDetectMode);

    /// Lead-off comparator threshold
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LeadOffCompThreshold {
        PositiveSide(CompPositiveSide),
        NegativeSide(CompNegativeSide),
    }

    impl From<LeadOffCompThreshold> for u8 {
        fn from(v: LeadOffCompThreshold) -> Self {
            match v {
                LeadOffCompThreshold::PositiveSide(vv) => vv as u8,
                LeadOffCompThreshold::NegativeSide(vv) => vv as u8,
            }
        }
    }

    /// Comparator positive side
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum CompPositiveSide {
        Pct_95_5 = 0b000,
        Pct_92_5 = 0b001,
        Pct_90_0 = 0b010,
        Pct_87_5 = 0b011,
        Pct_85_0 = 0b100,
        Pct_80_0 = 0b101,
        Pct_75_0 = 0b110,
        Pct_70_0 = 0b111,
    }

    /// Comparator negative side
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum CompNegativeSide {
        Pct_5_0 = 0b000,
        Pct_7_5 = 0b001,
        Pct_10_0 = 0b010,
        Pct_12_5 = 0b011,
        Pct_15_0 = 0b100,
        Pct_20_0 = 0b101,
        Pct_25_0 = 0b110,
        Pct_30_0 = 0b111,
    }

    // 0x04
    bitfield! {
        /// The lead-off control register configures the lead-off detection operation
        pub struct LeadOffControlReg(u8);
        impl Debug;

        /// Lead-off frequency
        ///
        /// These bits determine the frequency of lead-off detect for each channel.
        ///
        ///   - 00 = When any bits of the `LOFF_SENSP` or `LOFF_SENSN` registers are turned on,
        ///     make sure that `FLEAD`[1:0] are either set to 01 or 11
        ///   - 01 = AC lead-offdetection at `fDR`/ 4
        ///   - 10 = Do not use
        ///   - 11 = DC lead-off detection turned on
        ///
        pub flead_off, set_flead_off : 1, 0;

        /// Lead-off current magnitude
        ///
        /// These bits determine the magnitude of current for the
        /// current lead-off mode.
        ///   - 00 = 6 nA
        ///   - 01 = 12 nA
        ///   - 10 = 18 nA
        ///   - 11 = 24 nA
        ///
        pub ilead_off, set_ilead_off : 3, 2;

        /// Lead-off detection mode
        ///
        /// This bit determines the lead-off detection mode.
        ///   - 0 = Current source mode lead-off
        ///   - 1 = pullup or pulldown resistor mode lead-off
        ///
        pub vlead_off_en, set_vlead_off_en : 4;

        /// Lead-off comparator threshold
        ///
        /// Comparator positive side
        ///   - 000 = 95%
        ///   - 001 = 92.5%
        ///   - 010 = 90%
        ///   - 011 = 87.5%
        ///   - 100 = 85%
        ///   - 101 = 80%
        ///   - 110 = 75%
        ///   - 111 = 70%
        ///
        /// Comparator negative side
        ///   - 000 = 5%
        ///   - 001 = 7.5%
        ///   - 010 = 10%
        ///   - 011 = 12.5%
        ///   - 100 = 15%
        ///   - 101 = 20%
        ///   - 110 = 25%
        ///   - 111 = 30%
        ///
        pub comp_th, set_comp_th : 7, 5;
    }

    impl From<LeadOffControl> for LeadOffControlReg {
        fn from(param: LeadOffControl) -> Self {
            let mut reg = LeadOffControlReg(0);
            reg.set_flead_off(param.frequency as u8);
            reg.set_ilead_off(param.magnitude as u8);
            reg.set_vlead_off_en(param.detection_mode.into());
            reg.set_comp_th(param.comparator_threshold.into());
            reg
        }
    }

    impl TryFrom<LeadOffControlReg> for LeadOffControl {
        type Error = u8;

        fn try_from(reg: LeadOffControlReg) -> Result<Self, Self::Error> {
            Ok(LeadOffControl {
                frequency: LeadOffFreq::try_from(reg.flead_off()).map_err(|_| reg.0)?,
                magnitude: LeadOffMagnitude::try_from(reg.ilead_off()).map_err(|_| reg.0)?,
                detection_mode: LeadOffDetectMode::try_from(reg.vlead_off_en() as u8)
                    .map_err(|_| reg.0)?,
                comparator_threshold: LeadOffCompThreshold::PositiveSide(
                    CompPositiveSide::try_from(reg.flead_off()).map_err(|_| reg.0)?,
                ),
            })
        }
    }

    /// Lead-off sense setup
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LeadOffSense {
        pub ch1_enable: bool,
        pub ch2_enable: bool,
        pub ch3_enable: bool,
        pub ch4_enable: bool,
        pub ch5_enable: bool,
        pub ch6_enable: bool,
        pub ch7_enable: bool,
        pub ch8_enable: bool,
    }

    impl Default for LeadOffSense {
        fn default() -> Self {
            LeadOffSense {
                ch1_enable: false,
                ch2_enable: false,
                ch3_enable: false,
                ch4_enable: false,
                ch5_enable: false,
                ch6_enable: false,
                ch7_enable: false,
                ch8_enable: false,
            }
        }
    }

    // 0x0F-0x10
    bitfield! {
        /// LOFF_SENSP/N : Positive/Negative Signal Lead-Off Detection Register
        pub struct LeadOffSenseReg(u8);
        impl Debug;

        /// INxP/N leadoff
        ///
        /// Enable lead-off detection on INxP/N
        ///
        ///   - 0: Disabled
        ///   - 1: Enabled
        ///
        pub loff1, set_loff1 : 0;
        pub loff2, set_loff2 : 1;
        pub loff3, set_loff3 : 2;
        pub loff4, set_loff4 : 3;
        pub loff5, set_loff5 : 4;
        pub loff6, set_loff6 : 5;
        pub loff7, set_loff7 : 6;
        pub loff8, set_loff8 : 7;
    }

    impl From<LeadOffSense> for LeadOffSenseReg {
        fn from(param: LeadOffSense) -> Self {
            let mut reg = LeadOffSenseReg(0);
            reg.set_loff1(param.ch1_enable);
            reg.set_loff2(param.ch2_enable);
            reg.set_loff3(param.ch3_enable);
            reg.set_loff4(param.ch4_enable);
            reg.set_loff5(param.ch5_enable);
            reg.set_loff6(param.ch6_enable);
            reg.set_loff7(param.ch7_enable);
            reg.set_loff8(param.ch8_enable);
            reg
        }
    }

    impl TryFrom<LeadOffSenseReg> for LeadOffSense {
        type Error = u8;

        fn try_from(reg: LeadOffSenseReg) -> Result<Self, Self::Error> {
            Ok(LeadOffSense {
                ch1_enable: reg.loff1(),
                ch2_enable: reg.loff2(),
                ch3_enable: reg.loff3(),
                ch4_enable: reg.loff4(),
                ch5_enable: reg.loff5(),
                ch6_enable: reg.loff6(),
                ch7_enable: reg.loff7(),
                ch8_enable: reg.loff8(),
            })
        }
    }

    /// Controls the direction of the current used for lead-off derivation
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LeadOffFlip {
        /// Channel N polarity flip
        pub ch1_flip: bool,
        pub ch2_flip: bool,
        pub ch3_flip: bool,
        pub ch4_flip: bool,
        pub ch5_flip: bool,
        pub ch6_flip: bool,
        pub ch7_flip: bool,
        pub ch8_flip: bool,
    }

    impl Default for LeadOffFlip {
        fn default() -> Self {
            LeadOffFlip {
                ch1_flip: false,
                ch2_flip: false,
                ch3_flip: false,
                ch4_flip: false,
                ch5_flip: false,
                ch6_flip: false,
                ch7_flip: false,
                ch8_flip: false,
            }
        }
    }

    // 0x11
    bitfield! {
        /// LOFF_FLIP: Lead-Off Flip Register
        ///
        /// This register controls the direction of the current used for lead-off derivation.
        ///
        pub struct LeadOffFlipReg(u8);
        impl Debug;

        /// Channel X LOFF Polarity Flip
        ///
        /// Flip the pullup/pulldown polarity of the current source or
        /// resistor on channel N for lead-off derivation.
        ///
        ///   - 0: No Flip: INXP is pulled to `AVDD` and INXN pulled to `AVSS`
        ///   - 1: Flipped: INXP is pulled to `AVSS` and INXN pulled to `AVDD`
        ///
        pub flip1, set_flip1 : 0;
        pub flip2, set_flip2 : 1;
        pub flip3, set_flip3 : 2;
        pub flip4, set_flip4 : 3;
        pub flip5, set_flip5 : 4;
        pub flip6, set_flip6 : 5;
        pub flip7, set_flip7 : 6;
        pub flip8, set_flip8 : 7;
    }

    impl From<LeadOffFlip> for LeadOffFlipReg {
        fn from(param: LeadOffFlip) -> Self {
            let mut reg = LeadOffFlipReg(0);
            reg.set_flip1(param.ch1_flip);
            reg.set_flip2(param.ch2_flip);
            reg.set_flip3(param.ch3_flip);
            reg.set_flip4(param.ch4_flip);
            reg.set_flip5(param.ch5_flip);
            reg.set_flip6(param.ch6_flip);
            reg.set_flip7(param.ch7_flip);
            reg.set_flip8(param.ch8_flip);
            reg
        }
    }

    impl TryFrom<LeadOffFlipReg> for LeadOffFlip {
        type Error = u8;

        fn try_from(reg: LeadOffFlipReg) -> Result<Self, Self::Error> {
            Ok(LeadOffFlip {
                ch1_flip: reg.flip1(),
                ch2_flip: reg.flip2(),
                ch3_flip: reg.flip3(),
                ch4_flip: reg.flip4(),
                ch5_flip: reg.flip5(),
                ch6_flip: reg.flip6(),
                ch7_flip: reg.flip7(),
                ch8_flip: reg.flip8(),
            })
        }
    }
}

pub mod gpio {
    use super::*;

    /// GPIO configuration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Gpio {
        pub mode: [GpioMode; 4],
        pub data: [bool; 4],
    }

    impl Default for Gpio {
        fn default() -> Self {
            Gpio {
                mode: [GpioMode::Input; 4],
                data: [false; 4],
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum GpioMode {
        Output = 0b0,
        Input = 0b1,
    }
    impl_from_enum_to_bool!(GpioMode);

    // 0x14
    bitfield! {
        /// GPIO: General-Purpose I/O Register
        ///
        /// The general-purpose I/O register controls the action of the three GPIOpins.
        /// When `RESP_CTRL`[1:0] is in mode 01 and 11, the GPIO2, GPIO3,and GPIO4 pins are not
        /// available for use.
        ///
        pub struct GpioReg(u8);
        impl Debug;

        /// GPIO control (corresponding GPIOD)
        ///
        /// These bits determine if the corresponding GPIOD pin is an input or output.
        ///
        ///   - 0 = Output
        ///   - 1 = Input
        ///
        pub gpioc1, set_gpioc1 : 0;
        pub gpioc2, set_gpioc2 : 1;
        pub gpioc3, set_gpioc3 : 2;
        pub gpioc4, set_gpioc4 : 3;

        /// GPIO data
        ///
        /// These bits are used to read and write data to the GPIO ports. When reading the
        /// register, the data returned correspond to the state of the GPIO external pins, whether they are
        /// programmed as inputs or as outputs. As outputs, a write to the GPIOD sets the output value. As
        /// inputs, a write to the GPIOD has no effect. GPIO is not available in certain respiration modes.
        ///
        pub gpiod1, set_gpiod1 : 4;
        pub gpiod2, set_gpiod2 : 5;
        pub gpiod3, set_gpiod3 : 6;
        pub gpiod4, set_gpiod4 : 7;
    }

    impl From<Gpio> for GpioReg {
        fn from(param: Gpio) -> Self {
            let mut reg = GpioReg(0);
            reg.set_gpioc1(param.mode[0].into());
            reg.set_gpioc2(param.mode[1].into());
            reg.set_gpioc3(param.mode[2].into());
            reg.set_gpioc4(param.mode[3].into());

            reg.set_gpiod1(param.data[0]);
            reg.set_gpiod2(param.data[1]);
            reg.set_gpiod3(param.data[2]);
            reg.set_gpiod4(param.data[3]);
            reg
        }
    }

    impl TryFrom<GpioReg> for Gpio {
        type Error = u8;

        fn try_from(reg: GpioReg) -> Result<Self, Self::Error> {
            Ok(Gpio {
                mode: [
                    GpioMode::try_from(reg.gpioc1() as u8).map_err(|_| reg.0)?,
                    GpioMode::try_from(reg.gpioc2() as u8).map_err(|_| reg.0)?,
                    GpioMode::try_from(reg.gpioc3() as u8).map_err(|_| reg.0)?,
                    GpioMode::try_from(reg.gpioc4() as u8).map_err(|_| reg.0)?,
                ],
                data: [reg.gpiod1(), reg.gpiod2(), reg.gpiod3(), reg.gpiod4()],
            })
        }
    }
}
