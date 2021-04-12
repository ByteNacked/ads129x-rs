/// SPI commands
///
/// Table 13 page 35 of specification.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Command {
    /// Wake-up from standby mode
    WAKEUP    = 0x02,
    /// Enter standy mode
    STANDBY   = 0x04,
    /// Reset the device
    RESET     = 0x06,
    /// Start or restart (synchronize) conversions
    START     = 0x08,
    /// Stop conversion
    STOP      = 0x0A,
    /// Channel offset calibration
    OFFSETCAL = 0x1A,
    /// Enable Read Data Continuous Mode (default @ powerup)
    ///
    /// During this mode RREG commands are ignored.
    RDATAC    = 0x10,
    /// Stop Read Data Continuously Mode
    SDATAC    = 0x11,
    /// Read data by command; supports multiple read back
    RDATA     = 0x12,
    /// Read registers starting at an address
    RREG      = 0x20,
    /// Write registers starting at an address
    WREG      = 0x40,
}
