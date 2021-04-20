use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};

use ads129x::ads1292::chan::*;
use ads129x::ads1292::conf::*;
use ads129x::ads1292::resp::*;
// use ads129x::ads1292::gpio::*;
use ads129x::Ads129x;

struct MockNcs;

impl OutputPin for MockNcs {
    type Error = core::convert::Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct MockDelay;

impl DelayUs<u32> for MockDelay {
    fn delay_us(&mut self, _us: u32) {}
}

#[test]
fn test() {
    // Configure expectations
    let expectations = [
        // Stop data cont
        SpiTransaction::write(vec![0x11]),
        // Config1 250Hz
        SpiTransaction::write(vec![0x41, 0x00, 0b0000_0001]),
        // Config2
        SpiTransaction::write(vec![0x42, 0x00, 0b1010_0011]),
        // Chan1
        SpiTransaction::write(vec![0x44, 0x00, 0b0001_0000]),
        // Chan2
        SpiTransaction::write(vec![0x45, 0x00, 0b0100_0000]),
        // Resp1
        SpiTransaction::write(vec![0x49, 0x00, 0b1101_1110]),
    ];

    let ncs = MockNcs;

    let spi = SpiMock::new(&expectations);

    let mut ads1292 = Ads129x::new_ads1292(spi, ncs);
    ads1292.set_command_mode(MockDelay).unwrap();

    // Basic setup
    let config = Config {
        sample_rate: SampleRate::Sps250,
        ..Default::default()
    };
    ads1292.set_config(config, MockDelay).unwrap();

    let misc = MiscConfig {
        test_signal_freq: TestSignalFreq::SquareWave_1Hz,
        test_signal_enable: true,
        ref_buffer_enable: true,
        ..Default::default()
    };
    ads1292.set_misc_config(misc, MockDelay).unwrap();

    // Channel setup
    ads1292
        .set_chan_1(
            Chan::PowerUp {
                gain:  ChannelGain::X1,
                input: ChannelInput::Normal,
            },
            MockDelay,
        )
        .unwrap();
    ads1292
        .set_chan_2(
            Chan::PowerUp {
                gain:  ChannelGain::X4,
                input: ChannelInput::Normal,
            },
            MockDelay,
        )
        .unwrap();

    // Resp
    ads1292
        .set_resp(
            Resp1 {
                clock:               RespClock::Internal,
                phase:               RespPhase::RespPhase32kHz(RespPhase32kHz::Deg_78_75),
                modulation_enable:   true,
                demodulation_enable: true,
            },
            MockDelay,
        )
        .unwrap();

    // Finalize expectations
    let (mut spi, _) = ads1292.destroy();
    spi.done();
}
