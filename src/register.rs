use core::convert::TryFrom;

use bitfield::bitfield;
use num_enum::TryFromPrimitive;

pub mod id {
    use super::*;
    
    #[derive(Debug)]
    pub enum DevModel {
        Ads1291,
        Ads1292,
        Ads1292R,
        Ads1294,
        Ads1296,
        Ads1298,
        Ads1294R,
        Ads1296R,
        Ads1298R,
    }
    

    bitfield!{
        // 0x00
        pub struct IdReg(u8);
        impl Debug;
        pub channel_id, _ : 2, 0;
        pub reserved, _ : 4, 3;
        pub model_id, _ : 7, 5;
    }

    #[derive(Debug, Clone, Copy)]
    pub enum IdRegError {
        /// Should always equals to 0b10
        ReservedFieldMismatch(u8),
        Unsupported(u8),
    }
    
    impl core::convert::TryFrom<IdReg> for DevModel {
        type Error = IdRegError;

        fn try_from(idreg: IdReg) -> Result<Self, Self::Error> {
            // Mismatched reserved bits
            if idreg.reserved() != 0b10 {
                return Err(IdRegError::ReservedFieldMismatch(idreg.0));
            }

            Ok(match (idreg.channel_id(), idreg.model_id()) {
                // 4-8Ch
                (0b000, 0b100) => DevModel::Ads1294,
                (0b001, 0b100) => DevModel::Ads1296,
                (0b010, 0b100) => DevModel::Ads1298,
                // 4-8Ch R
                (0b000, 0b110) => DevModel::Ads1294R,
                (0b001, 0b110) => DevModel::Ads1296R,
                (0b010, 0b110) => DevModel::Ads1298R,
                // 1-2Ch
                (0b10, 0b010) => DevModel::Ads1291,
                (0b11, 0b010) => DevModel::Ads1292,
                (0b11, 0b011) => DevModel::Ads1292R,

                _ => return Err(IdRegError::Unsupported(idreg.0)),
            })
        }
    }
}

































