use bitfield::bitfield;

pub trait DataFrame<'frame> {
    const CHAN_NUM: usize;
    const CHAN_WIDTH_BITS: usize;

    fn set_leadoff_status() -> &'frame mut [bool];
    fn gpio_status() -> &'frame mut [bool];
    fn channel_data() -> &'frame mut [i32];
}

bitfield! {
    pub struct DataStatusWord(u32);
    impl Debug;

    pub sync, set_sync : 3, 0;
    pub loff_statp, set_loff_statp : 11, 4;
    pub loff_statn, set_loff_statn : 19, 12;
    pub gpio, set_gpio : 23, 20;
}

#[repr(C, packed)]
pub struct DataFrameCh<CHW, const CH_NUM: usize> {
    pub status:  [u8; 3],
    pub ch_data: [CHW; CH_NUM],
}
