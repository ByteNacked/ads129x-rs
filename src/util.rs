macro_rules! impl_cmd {
    (__INNER: $doc:expr, $fn_name:ident, $command:ident) => {
        #[doc = $doc]
        pub fn $fn_name(&mut self, delay: impl DelayUs<u32>) -> Ads129xResult<(), E> {
            self.spi.write(&[command::Command::$command as u8], delay)?;
            Ok(())
        }
    };
    ($fn_name:ident, $command:ident) => {
        impl_cmd!(
            __INNER: concat!("Spi command ", stringify!($command)),
            $fn_name,
            $command
        );
    };
}

macro_rules! write_reg {
    (_INNER: $doc:expr, FAM: $family_path:ident, FN: $fn_name:ident, REG: $reg_name:ident ($param_path:ident::$param_ty:ident => $reg_path:ident::$reg_ty:ident)) => {
        #[doc = $doc]
        pub fn $fn_name(
            &mut self,
            param: $family_path::$param_path::$param_ty,
            delay: impl DelayUs<u32>,
        ) -> Ads129xResult<(), E> {
            let mut words = [
                command::Command::WREG as u8 | $family_path::Register::$reg_name as u8,
                0x00,
                $family_path::$reg_path::$reg_ty::from(param).0,
            ];
            let _ = self.spi.write(&mut words, delay)?;
            Ok(())
        }
    };
    (FAM: $family_path:ident, FN: $fn_name:ident, REG: $reg_name:ident ($param_path:ident::$param_ty:ident => $reg_path:ident::$reg_ty:ident)) => {
        write_reg!(
            _INNER: concat!("Write register ", stringify!($reg_name)),
            FAM: $family_path,
            FN: $fn_name,
            REG: $reg_name ($param_path::$param_ty => $reg_path::$reg_ty)
        );
    };
}

macro_rules! read_reg {
    (_INNER: $doc:expr, FAM: $family_path:ident, FN: $fn_name:ident, REG: $reg_name:ident ($param_path:ident::$param_ty:ident <= $reg_path:ident::$reg_ty:ident)) => {
        #[doc = $doc]
        pub fn $fn_name(
            &mut self,
            delay: impl DelayUs<u32>,
        ) -> Ads129xResult<$family_path::$param_path::$param_ty, E> {
            let mut words = [
                command::Command::RREG as u8 | $family_path::Register::$reg_name as u8,
                0x00,
                0xA5,
            ];
            let res = self.spi.transfer(&mut words, delay)?;

            let param = $family_path::$param_path::$param_ty::try_from(
                $family_path::$reg_path::$reg_ty(res[2]),
            )
            .map_err(|e| Ads129xError::ReadInterpret(e))?;

            Ok(param)
        }
    };
    (FAM: $family_path:ident, FN: $fn_name:ident, REG: $reg_name:ident ($param_path:ident::$param_ty:ident <= $reg_path:ident::$reg_ty:ident)) => {
        read_reg!(
            _INNER: concat!("Read register ", stringify!($reg_name)),
            FAM: $family_path,
            FN: $fn_name,
            REG: $reg_name ($param_path::$param_ty <= $reg_path::$reg_ty)
        );
    };
}
