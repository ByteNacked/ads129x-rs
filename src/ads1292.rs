#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

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
    ID        = 0x00,
    /// Configuration Register 1
    CONFIG1   = 0x01,
    /// Configuration Register 2
    CONFIG2   = 0x02,
    /// Lead-Off Control Register
    LOFF      = 0x03,
    /// Channel 1 Settings
    CH1SET    = 0x04,
    /// Channel 2 Settings
    CH2SET    = 0x05,
    /// Right Leg Drive Sense Selection
    RLD_SENS  = 0x06,
    /// Lead-Off Sense Selection
    LOFF_SENS = 0x07,
    /// Lead-Off Status
    LOFF_STAT = 0x08,
    /// Respiration Control Register 1
    RESP1     = 0x09,
    /// Respiration Control Register 2    
    RESP2     = 0x0A,
    /// General-Purpose I/O Register
    GPIO      = 0x0B,
}

pub mod conf {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Config {
        pub mode:        Mode,
        pub sample_rate: SampleRate,
    }

    impl Default for Config {
        fn default() -> Self {
            Config {
                mode:        Mode::Continuous,
                sample_rate: SampleRate::Sps500,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum Mode {
        Continuous = 0x00,
        SingleShot = 0x01,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum SampleRate {
        Sps125 = 0b000,
        Sps250 = 0b001,
        Sps500 = 0b010,
        KSps1  = 0b011,
        KSps2  = 0b100,
        KSps4  = 0b101,
        KSps8  = 0b110,
    }

    impl Default for SampleRate {
        fn default() -> Self {
            SampleRate::Sps500
        }
    }

    // 0x01
    bitfield! {
        /// Configuration for the register that configures each ADC channel sample rate.
        pub struct Config1Reg(u8);
        impl Debug;
        /// The oversampling rate used by all channels.
        pub oversampling, set_oversampling: 2, 0;
        /// The single shot conversion mode, otherwise use a continuous conversion mode.
        pub single_shot, set_single_shot: 7;
    }

    impl From<Config> for Config1Reg {
        fn from(config: Config) -> Self {
            let mut reg = Config1Reg(0);
            reg.set_single_shot(if config.mode == Mode::SingleShot {
                true
            } else {
                false
            });
            reg.set_oversampling(config.sample_rate as u8);
            reg
        }
    }

    impl TryFrom<Config1Reg> for Config {
        type Error = u8;

        fn try_from(reg: Config1Reg) -> Result<Self, Self::Error> {
            Ok(Config {
                mode:        Mode::try_from(reg.single_shot() as u8).map_err(|_| reg.0)?,
                sample_rate: SampleRate::try_from(reg.oversampling()).map_err(|_| reg.0)?,
            })
        }
    }

    /// Various configurations
    pub struct MiscConfig {
        /// Test signal frequency
        pub test_signal_freq:          TestSignalFreq,
        /// Test signal enable flag
        pub test_signal_enable:        bool,
        /// Oscillator clock output
        pub osc_clock_output:          bool,
        /// Reference voltage `VREF` 4V enable
        /// or 2V if disabled
        pub vref_4V_enable:            bool,
        /// Power-down reference buffer enable
        pub ref_buffer_enable:         bool,
        /// Lead-off comparator enable
        pub leadoff_comparator_enable: bool,
    }

    impl Default for MiscConfig {
        fn default() -> Self {
            MiscConfig {
                test_signal_freq:          TestSignalFreq::AtDc,
                test_signal_enable:        false,
                osc_clock_output:          false,
                vref_4V_enable:            false,
                ref_buffer_enable:         false,
                leadoff_comparator_enable: false,
            }
        }
    }

    /// Test signal frequency
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum TestSignalFreq {
        /// At dc
        AtDc           = 0x00,
        /// Squarewaveat 1 Hz
        SquareWave_1Hz = 0x01,
    }
    impl_from_enum_to_bool!(TestSignalFreq);

    // 0x02
    bitfield! {
        /// Configuration for the register that configures the test signal, clock, reference and LOFF buffer.
        pub struct Config2Reg(u8);
        impl Debug;
        /// Determines the test signal frequency.
        pub test_freq, set_test_freq: 0;
        /// Determines whether the test signal is turned on or off.
        pub int_test, set_int_test: 1;
        /// Determines if the internal oscillator signal is connected to the CLK pin when an internal oscillator is used.
        pub clk_en, set_clk_en: 3;
        /// Enable 4.033v reference, otherwise use the 2.42v reference.
        pub vref_4v, set_vref_4v: 4;
        /// Powers down the internal reference buffer so that the external reference can be used.
        pub pdb_refbuf, set_pdb_refbuf: 5;
        /// Power down the lead-off comparators.
        pub pdb_loff_comp, set_pdb_loff_comp: 6;
    }

    impl From<MiscConfig> for Config2Reg {
        fn from(param: MiscConfig) -> Self {
            let mut reg = Config2Reg(0x80);
            reg.set_test_freq(param.test_signal_freq.into());
            reg.set_int_test(param.test_signal_enable);
            reg.set_clk_en(param.osc_clock_output);
            reg.set_vref_4v(param.vref_4V_enable);
            reg.set_pdb_refbuf(param.ref_buffer_enable);
            reg.set_pdb_loff_comp(param.leadoff_comparator_enable);
            reg
        }
    }

    impl TryFrom<Config2Reg> for MiscConfig {
        type Error = u8;

        fn try_from(reg: Config2Reg) -> Result<Self, Self::Error> {
            Ok(MiscConfig {
                test_signal_freq:          TestSignalFreq::try_from(reg.test_freq() as u8)
                    .map_err(|_| reg.0)?,
                test_signal_enable:        reg.int_test(),
                osc_clock_output:          reg.clk_en(),
                vref_4V_enable:            reg.vref_4v(),
                ref_buffer_enable:         reg.pdb_refbuf(),
                leadoff_comparator_enable: reg.pdb_loff_comp(),
            })
        }
    }
}

pub mod loff {
    use super::*;

    /// Lead-off control configuration
    pub struct LeadOffControl {
        pub frequency:            LeadOffFreq,
        pub magnitude:            LeadOffCurrentMagnitude,
        pub comparator_threshold: LeadOffCompThreshold,
    }

    /// Lead-off frequency
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum LeadOffFreq {
        /// DC lead-off detection turned on
        DC = 0b0,
        /// AC lead-offdetection at `fDR`/ 4
        AC = 0b1,
    }
    impl_from_enum_to_bool!(LeadOffFreq);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum LeadOffCurrentMagnitude {
        nA_6  = 0b00,
        nA_22 = 0b01,
        uA_6  = 0b10,
        uA_22 = 0b11,
    }

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
        Pct_5_0  = 0b000,
        Pct_7_5  = 0b001,
        Pct_10_0 = 0b010,
        Pct_12_5 = 0b011,
        Pct_15_0 = 0b100,
        Pct_20_0 = 0b101,
        Pct_25_0 = 0b110,
        Pct_30_0 = 0b111,
    }

    // 0x03
    bitfield! {
        /// Configuration for the register that configures the lead-off detection operation.
        pub struct LeadOffControlReg(u8);
        impl Debug;
        /// Selects ac (true) or dc (false) lead-off
        pub flead_off, set_flead_off: 0;
        /// Powers down the internal reference buffer so that the external reference can be used.
        pub ilead_off, set_ilead_off: 3, 2;
        /// Power down the lead-off comparators.
        pub comp_th, set_comp_th: 7, 5;
    }

    impl From<LeadOffControl> for LeadOffControlReg {
        fn from(param: LeadOffControl) -> Self {
            let mut reg = LeadOffControlReg(0);
            reg.set_flead_off(param.frequency.into());
            reg.set_ilead_off(param.magnitude as u8);
            reg.set_comp_th(param.comparator_threshold.into());
            reg
        }
    }

    impl TryFrom<LeadOffControlReg> for LeadOffControl {
        type Error = u8;

        fn try_from(reg: LeadOffControlReg) -> Result<Self, Self::Error> {
            Ok(LeadOffControl {
                frequency:            LeadOffFreq::try_from(reg.flead_off() as u8)
                    .map_err(|_| reg.0)?,
                magnitude:            LeadOffCurrentMagnitude::try_from(reg.ilead_off())
                    .map_err(|_| reg.0)?,
                comparator_threshold: LeadOffCompThreshold::PositiveSide(
                    CompPositiveSide::try_from(reg.comp_th()).map_err(|_| reg.0)?,
                ),
            })
        }
    }

    // 0x07
    bitfield! {
        /// Configuration for the register that selects the positive and negative side from each channel for lead-off detection.
        pub struct LoffSense(u8);

        /// Controls the direction of the current used for lead-off derivation for channel 2
        pub flip2, set_flip2: 5;
        /// Controls the direction of the current used for lead-off derivation for channel 1
        pub flip1, set_flip1: 4;
        /// Controls the selection of negative input from channel 2 for lead-off detection
        pub loff2n, set_loff2n: 3;
        /// Controls the selection of positive input from channel 2 for lead-off detection
        pub loff2p, set_loff2p: 2;
        /// Controls the selection of negative input from channel 1 for lead-off detection
        pub loff1n, set_loff1n: 1;
        /// Controls the selection of positive input from channel 1 for lead-off detection
        pub loff1p, set_loff1p: 0;
    }
    
    // Lead-Off status
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LeadOffStatus {
        pub ch1_positive_leadoff: bool,
        pub ch1_negative_leadoff: bool,
        pub ch2_positive_leadoff: bool,
        pub ch2_negative_leadoff: bool,
        pub rld_leadoff: bool,
        pub clk_div: ClkDiv,
    }

    impl Default for LeadOffStatus {
        fn default() -> Self {
            LeadOffStatus {
                ch1_positive_leadoff: false,
                ch1_negative_leadoff: false,
                ch2_positive_leadoff: false,
                ch2_negative_leadoff: false,
                rld_leadoff: false,
                clk_div: ClkDiv::Div4,
            }

        }
    }
    
    /// Clock divider selection
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ClkDiv {
        Div4 = 0x00,
        Div16 = 0x01,
    }
    impl_from_enum_to_bool!(ClkDiv);

    // 0x08
    bitfield! {
        /// Lead-Off Status register
        ///
        /// This register stores the status of whether the positive or negative electrode on each
        /// channel is on or off
        ///
        pub struct LeadOffStatusReg(u8);
        impl Debug;
        
        // TODO: Doc
        pub in1p_off, set_in1p_off: 0;
        pub in1n_off, set_in1n_off: 1;
        pub in2p_off, set_in2p_off: 2;
        pub in2n_off, set_in2n_off: 3;
        pub rld_stat, set_rld_stat: 4;
        pub clk_div, set_clk_div: 6;
    }
    
    impl From<LeadOffStatus> for LeadOffStatusReg {
        fn from(param: LeadOffStatus) -> Self {
            let mut reg = LeadOffStatusReg(0);
            // Only clk_div writable
            reg.set_clk_div(param.clk_div.into());
            reg
        }
    }

    impl TryFrom<LeadOffStatusReg> for LeadOffStatus {
        type Error = u8;

        fn try_from(reg: LeadOffStatusReg) -> Result<Self, Self::Error> {
            Ok(LeadOffStatus{
                ch1_positive_leadoff: reg.in1p_off(),
                ch1_negative_leadoff: reg.in1n_off(),
                ch2_positive_leadoff: reg.in2p_off(),
                ch2_negative_leadoff: reg.in2n_off(),
                rld_leadoff: reg.rld_stat(),
                clk_div: ClkDiv::try_from(reg.clk_div() as u8).map_err(|_| reg.0)?,
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
            gain:  ChannelGain,
        },
        PowerDown,
    }

    impl Default for Chan {
        fn default() -> Self {
            Chan::PowerUp {
                input: ChannelInput::Normal,
                gain:  ChannelGain::X6,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ChannelInput {
        /// Normal electrode input (default)
        Normal            = 0b0000,
        /// Input shorted (for offset measurements)
        Shorted           = 0b0001,
        /// `RLD_MEASURE`
        Rld               = 0b0010,
        /// MVDD for supply measurement
        MVDD              = 0b0011,
        /// Temperature sensor
        TemperatureSensor = 0b0100,
        /// Test signal
        TestSig           = 0b0101,
        /// `RLD_DRP` (positive input is connected to `RLDIN`)
        RldDrp            = 0b0110,
        /// `RLD_DRM` (negative input is connected to `RLDIN`)
        RldDrm            = 0b0111,
        /// `RLD_DRPM` (both positive and negative inputs are connected to
        /// `RLDIN`)
        RldDrpm           = 0b1000,
        /// Route `IN3P` and `IN3N` to channel 1 inputs
        Channel3          = 0b1001,
    }

    /// PGA gain
    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ChannelGain {
        X6  = 0b000,
        X1  = 0b001,
        X2  = 0b010,
        X3  = 0b011,
        X4  = 0b100,
        X8  = 0b101,
        X12 = 0b110,
    }

    // 0x04-0x05
    bitfield! {
        /// Configuration for the register that configures the power mode, PGA gain, and multiplexer settings channels.
        pub struct ChanSetReg(u8);
        impl Debug;
        /// Determines the channel input selection.
        pub mux, set_mux: 3, 0;
        /// Determines the PGA gain setting for the channel.
        pub gain, set_gain: 6, 4;
        /// Power down the channel.
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
                    gain:  ChannelGain::try_from(reg.gain()).map_err(|_| reg.0)?,
                }
            })
        }
    }
}

pub mod resp {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Resp1 {
        pub clock:               RespClock,
        pub phase:               RespPhase,
        pub modulation_enable:   bool,
        pub demodulation_enable: bool,
    }

    impl Default for Resp1 {
        fn default() -> Self {
            Resp1 {
                clock:               RespClock::Internal,
                phase:               RespPhase::RespPhase32kHz(RespPhase32kHz::Deg_0),
                modulation_enable:   false,
                demodulation_enable: false,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum RespClock {
        Internal = 0x00,
        External = 0x01,
    }
    impl_from_enum_to_bool!(RespClock);

    #[derive(Debug, Clone, Copy, Eq)]
    #[repr(u8)]
    pub enum RespPhase {
        RespPhase32kHz(RespPhase32kHz),
        RespPhase64kHz(RespPhase64kHz),
    }

    impl From<RespPhase> for u8 {
        fn from(v: RespPhase) -> Self {
            match v {
                RespPhase::RespPhase32kHz(vv) => vv as u8,
                RespPhase::RespPhase64kHz(vv) => vv as u8,
            }
        }
    }

    impl PartialEq for RespPhase {
        fn eq(&self, other: &Self) -> bool {
            u8::from(*self) == u8::from(*other)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum RespPhase32kHz {
        Deg_0      = 0b0000,
        Deg_11_25  = 0b0001,
        Deg_22_5   = 0b0010,
        Deg_33_75  = 0b0011,
        Deg_45     = 0b0100,
        Deg_56_25  = 0b0101,
        Deg_67_5   = 0b0110,
        Deg_78_75  = 0b0111,
        Deg_90     = 0b1000,
        Deg_101_25 = 0b1001,
        Deg_112_5  = 0b1010,
        Deg_123_75 = 0b1011,
        Deg_135    = 0b1100,
        Deg_146_25 = 0b1101,
        Deg_157_5  = 0b1110,
        Deg_168_75 = 0b1111,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum RespPhase64kHz {
        Deg_0     = 0b0000,
        Deg_22_5  = 0b0001,
        Deg_45    = 0b0010,
        Deg_67_5  = 0b0011,
        Deg_90    = 0b0100,
        Deg_112_5 = 0b0101,
        Deg_135   = 0b0110,
        Deg_157_5 = 0b0111,
    }

    // 0x09
    bitfield! {
        /// Configuration for the register that controls the respiration and calibration functionality.
        pub struct RespControl1Reg(u8);
        impl Debug;

        /// Respiration control
        ///
        /// This bit sets the mode of the respiration circuitry.
        ///   - 0 = Internal respiration with internal clock
        ///   - 1 = Internal respiration with external clock
        ///
        pub resp_ctrl, set_resp_ctrl: 0;

        /// Not used
        ///
        /// Must be set 1
        _, set_must_set_1: 1;

        /// Respiraton phase
        ///
        /// These bits control the phase of the respiration demodulation control signal.
        ///
        /// |  bits  | RESP_CLK = 32kHz | RESP_CLK = 64kHZ |
        /// |--------|------------------|------------------|
        /// | 0b0000 | 0°               | 0                |
        /// | 0b0001 | 11.25°           | 22.5             |
        /// | 0b0010 | 22.5°            | 45               |
        /// | 0b0011 | 33.75            | 67.5             |
        /// | 0b0100 | 45               | 90               |
        /// | 0b0101 | 56.25            | 112.5            |
        /// | 0b0110 | 67.5             | 135              |
        /// | 0b0111 | 78.75            | 157.5            |
        /// | 0b1000 | 90               | NA               |
        /// | 0b1001 | 101.25           | NA               |
        /// | 0b1010 | 112.5            | NA               |
        /// | 0b1011 | 123.75           | NA               |
        /// | 0b1100 | 135              | NA               |
        /// | 0b1101 | 146.25           | NA               |
        /// | 0b1110 | 157.5            | NA               |
        /// | 0b1111 | 168.75           | NA               |
        ///
        pub resp_ph, set_resp_ph: 5, 2;

        /// Enables respiration modulation circuitry
        ///
        /// This bit enables and disables the modulation circuitry on channel 1.
        ///   - 0 = `RESP` modulation circuitry turned off (default)
        ///   - 1 = `RESP` modulation circuitry turned on
        ///
        pub resp_mod_en, set_resp_mod_en: 6;

        /// Enables respiration demodulation circuitry
        ///
        /// This bit enables and disables the demodulation circuitry on channel 1.
        ///   - 0 = `RESP` demodulation circuitry turned off (default)
        ///   - 1 = `RESP` demodulation circuitry turned on
        ///
        pub resp_demod_en, set_resp_demod_en: 7;
    }

    impl From<Resp1> for RespControl1Reg {
        fn from(param: Resp1) -> Self {
            let mut reg = RespControl1Reg(0x00);
            reg.set_resp_ctrl(param.clock.into());
            reg.set_must_set_1(true);
            reg.set_resp_ph(param.phase.into());
            reg.set_resp_mod_en(param.modulation_enable);
            reg.set_resp_demod_en(param.demodulation_enable);
            reg
        }
    }

    impl TryFrom<RespControl1Reg> for Resp1 {
        type Error = u8;

        fn try_from(reg: RespControl1Reg) -> Result<Self, Self::Error> {
            Ok(Resp1 {
                clock:               RespClock::try_from(reg.resp_ctrl() as u8)
                    .map_err(|_| reg.0)?,
                phase:               RespPhase::RespPhase32kHz(
                    RespPhase32kHz::try_from(reg.resp_ph()).map_err(|_| reg.0)?,
                ),
                modulation_enable:   reg.resp_mod_en(),
                demodulation_enable: reg.resp_demod_en(),
            })
        }
    }

    // 0x0A
    bitfield! {
        /// Configuration for the register that controls the respiration and calibration functionality.
        pub struct RespControl2Reg(u8);
        impl Debug;
        /// Determines the RLDREF signal source.
        /// Can be fed externally (false : 0) or internally by using (AVDD – AVSS) / 2 (true : 1).
        pub rldref_int, set_rldref_int: 1;
        /// Controls the respiration control frequency when RESP_CTRL = 0.
        ///
        /// **Warning**: this bit must be written with '1' for the ADS1291 and ADS1292.
        pub resp_freq_64khz, set_resp_freq_64khz: 2;
        /// Enables offset calibration
        pub calib_on, set_calib_on: 7;
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ChopFrequency {
    FmodDiv16 = 0b00,
    FmodDiv2  = 0b10,
    FmodDiv4  = 0b11,
    Unknown   = 0b01,
}

bitfield! {
    /// Configuration for the register that controls the selection of the positive and negative signals from each channel for right leg drive derivation.
    pub struct RLDSenseSelection(u8);

    /// Determines the PGA chop frequency.
    pub chop, set_chop: 7, 6;
    /// Enable the RLD buffer power.
    pub pdb_rld, set_pbd_rld: 5;
    /// Enable the RLD lead-off sense function.
    pub rld_loff_sense, set_rld_loff_sense: 4;

    /// Controls the selection of negative inputs from channel 2 for right leg drive derivation.
    pub rld2n, set_rld2n: 3;
    /// Controls the selection of positive inputs from channel 2 for right leg drive derivation.
    pub rld2p, set_rld2p: 2;
    /// Controls the selection of negative inputs from channel 1 for right leg drive derivation.
    pub rld1n, set_rld1n: 1;
    /// Controls the selection of positive inputs from channel 1 for right leg drive derivation.
    pub rld1p, set_rld1p: 0;
}
