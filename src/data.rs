use bitfield::bitfield;
use core::mem::size_of;

bitfield! {
    pub struct DataStatusWord(u32);
    impl Debug;

    pub u8, sync, set_sync : 23, 20;
    pub u8, loff_statp, set_loff_statp : 19, 12;
    pub u8, loff_statn, set_loff_statn : 11, 4;
    pub u8, gpio, set_gpio : 3, 0;
}

pub struct DataFrame<const CH: usize> {
    pub status_word: [u8; 3],
    pub data:        [i32; CH],
}

impl<const CH: usize> DataFrame<CH> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status_word(&self) -> DataStatusWord {
        // Big-endian-ish
        DataStatusWord(
            (self.status_word[0] as u32) << 2 * 8
                | (self.status_word[1] as u32) << 1 * 8
                | (self.status_word[2] as u32) << 0 * 8,
        )
    }
}

impl<const CH: usize> DataFrame<CH> {
    pub fn as_bytes(&self) -> &[u8] {
        // #SAFETY
        // It's safe to recast C, packed struct as bytes
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
}

impl<const CH: usize> Default for DataFrame<CH> {
    fn default() -> Self {
        DataFrame {
            status_word: [0; 3],
            data:        [0; CH],
        }
    }
}

impl<const CH: usize> core::fmt::Debug for DataFrame<CH> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut dbg_struct = f.debug_struct("DataFrame");
        for _ in 0..CH {
            dbg_struct.field("ch: ", &{
                let v = self.data[0];
                v
            });
        }

        Ok(())
    }
}
