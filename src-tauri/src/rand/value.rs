mod sealed {
    pub trait Value {
        const WIDTH: usize;
        fn decode(bytes: &[u8]) -> Self;
    }
}

/// Integer types whose complete bit patterns are valid random values.
pub trait RandomValue: sealed::Value + Copy + Default {}

macro_rules! integer {
    ($($type:ty),+ $(,)?) => {$(
        impl sealed::Value for $type {
            const WIDTH: usize = size_of::<Self>();
            fn decode(bytes: &[u8]) -> Self {
                let mut array = [0; size_of::<Self>()];
                array.copy_from_slice(bytes);
                Self::from_le_bytes(array)
            }
        }
        impl RandomValue for $type {}
    )+};
}

integer!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
