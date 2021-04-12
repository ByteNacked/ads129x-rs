use bitfield::bitfield;
use core::mem::size_of;

pub trait DataFrameTrait<'frame> {
    const CHAN_NUM: usize;
    const CHAN_WIDTH_BITS: usize;

    fn set_leadoff_status() -> &'frame mut [bool];
    fn gpio_status() -> &'frame mut [bool];
    fn channel_data() -> &'frame mut [i32];
}

bitfield! {
    pub struct DataStatusWord(u32);
    impl Debug;

    pub u8, sync, set_sync : 3, 0;
    pub u8, loff_statp, set_loff_statp : 11, 4;
    pub u8, loff_statn, set_loff_statn : 19, 12;
    pub u8, gpio, set_gpio : 23, 20;
}

#[repr(C, packed)]
pub struct DataFrame<const CH: usize> {
    pub status_word: [u8; 3],
    pub ch_data:     [i32; CH],
}

impl<const CH: usize> DataFrame<CH> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status_word(&self) -> DataStatusWord {
        DataStatusWord(
            (self.status_word[0] as u32) << 0 * 8
                | (self.status_word[1] as u32) << 1 * 8
                | (self.status_word[2] as u32) << 2 * 8,
        )
    }
}

impl<const CH: usize> DataFrame<CH> {
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        // #SAFETY
        // It's safe to recast C, packed struct as bytes
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}

impl<const CH: usize> Default for DataFrame<CH> {
    fn default() -> Self {
        DataFrame {
            status_word: [0; 3],
            ch_data:     [0; CH],
        }
    }
}
