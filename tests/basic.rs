use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};

use ads129x::ads1298::chan::*;
use ads129x::ads1298::conf::*;
use ads129x::ads1298::gpio::*;
use ads129x::ads1298::loff::*;
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
        // Config1
        SpiTransaction::write(vec![0x41, 0x00, 0b0110_0100]),
        // Config2
        SpiTransaction::write(vec![0x42, 0x00, 0b0001_0101]),
        // Config3
        SpiTransaction::write(vec![0x43, 0x00, 0b1100_0000]),
        // CHNSET
        SpiTransaction::write(vec![0x45, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x46, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x47, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x48, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x49, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x4A, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x4B, 0x00, 0b0100_0000]),
        SpiTransaction::write(vec![0x4C, 0x00, 0b0100_0000]),
        // GPIO
        SpiTransaction::write(vec![0x54, 0x00, 0b0000_0000]),
        // Lead-off
        SpiTransaction::write(vec![0x4f, 0x00, 0b0111_0111]),
        SpiTransaction::write(vec![0x50, 0x00, 0b1000_0011]),
        SpiTransaction::write(vec![0x51, 0x00, 0b1000_0100]),
        SpiTransaction::write(vec![0x44, 0x00, 0b0000_1111]),
        // Config 4
        SpiTransaction::write(vec![0x57, 0x00, 0b0000_0010]),
        // SpiTransaction::transfer(vec![3, 4], vec![5, 6]),
    ];

    let ncs = MockNcs;

    let spi = SpiMock::new(&expectations);

    let mut ads1298 = Ads129x::new_ads1298(spi, ncs);
    ads1298.set_command_mode(MockDelay).unwrap();

    // Basic setup

    let config = Config {
        mode:             Mode::LowPower(SampleRateLP::KSps1),
        osc_clock_output: true,
        daisy_chain:      false,
    };
    ads1298.set_config(config, MockDelay).unwrap();

    let ts_config = TestSignalConfig {
        frequency: TestSignalFreq::PulsedAtFclk_div_2_20,
        amplitude: TestSignalAmp::Mode_x2,
        source: TestSignalSource::Internal,
        ..Default::default()
    };
    ads1298
        .set_test_signal_config(ts_config, MockDelay)
        .unwrap();

    let rld_config = RldConfig {
        ref_buffer_enable: true,
        ..Default::default()
    };
    ads1298.set_rld_config(rld_config, MockDelay).unwrap();

    // Channel setup

    let chan = Chan::PowerUp {
        gain:  ChannelGain::X4,
        input: ChannelInput::Normal,
    };
    ads1298.set_chan_1(chan, MockDelay).unwrap();
    ads1298.set_chan_2(chan, MockDelay).unwrap();
    ads1298.set_chan_3(chan, MockDelay).unwrap();
    ads1298.set_chan_4(chan, MockDelay).unwrap();
    ads1298.set_chan_5(chan, MockDelay).unwrap();
    ads1298.set_chan_6(chan, MockDelay).unwrap();
    ads1298.set_chan_7(chan, MockDelay).unwrap();
    ads1298.set_chan_8(chan, MockDelay).unwrap();

    ads1298
        .set_gpio(
            Gpio {
                mode: [GpioMode::Output; 4],
                data: [false; 4],
            },
            MockDelay,
        )
        .unwrap();

    // Lead-off setup

    ads1298
        .set_leadoff_sense_positive(
            LeadOffSense {
                ch1_enable: true,
                ch2_enable: true,
                ch3_enable: true,
                ch4_enable: false,
                ch5_enable: true,
                ch6_enable: true,
                ch7_enable: true,
                ch8_enable: false,
            },
            MockDelay,
        )
        .unwrap();

    ads1298
        .set_leadoff_sense_negative(
            LeadOffSense {
                ch1_enable: true,
                ch2_enable: true,
                ch3_enable: false,
                ch4_enable: false,
                ch5_enable: false,
                ch6_enable: false,
                ch7_enable: false,
                ch8_enable: true,
            },
            MockDelay,
        )
        .unwrap();

    ads1298
        .set_leadoff_flip(
            LeadOffFlip {
                ch1_flip: false,
                ch2_flip: false,
                ch3_flip: true,
                ch4_flip: false,
                ch5_flip: false,
                ch6_flip: false,
                ch7_flip: false,
                ch8_flip: true,
            },
            MockDelay,
        )
        .unwrap();

    ads1298
        .set_leadoff_control(
            LeadOffControl {
                frequency: LeadOffFreq::DC,
                magnitude: LeadOffMagnitude::nA_24,
                ..Default::default()
            },
            MockDelay,
        )
        .unwrap();

    // Config 4
    ads1298
        .set_misc_config(
            MiscConfig {
                leadoff_comparator_enable: true,
                ..Default::default()
            },
            MockDelay,
        )
        .unwrap();

    // Finalize expectations
    let (mut spi, _) = ads1298.destroy();
    spi.done();
}
