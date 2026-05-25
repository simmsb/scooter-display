#[rustfmt::skip]
#[derive(minicbor::Encode, minicbor::Decode, minicbor::CborLen, PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
#[repr(u8)]
pub enum PinDigit {
    #[default]
    #[n(0)] D0,
    #[n(1)] D1,
    #[n(2)] D2,
    #[n(3)] D3,
    #[n(4)] D4,
    #[n(5)] D5,
    #[n(6)] D6,
    #[n(7)] D7,
    #[n(8)] D8,
    #[n(9)] D9,
}

impl PinDigit {
    pub fn as_char(self) -> char {
        char::from_digit(self as u32, 10).unwrap()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PinDigit::D0 => "0",
            PinDigit::D1 => "1",
            PinDigit::D2 => "2",
            PinDigit::D3 => "3",
            PinDigit::D4 => "4",
            PinDigit::D5 => "5",
            PinDigit::D6 => "6",
            PinDigit::D7 => "7",
            PinDigit::D8 => "8",
            PinDigit::D9 => "9",
        }
    }

    pub fn next(self) -> Self {
        use PinDigit::*;

        match self {
            D0 => D1,
            D1 => D2,
            D2 => D3,
            D3 => D4,
            D4 => D5,
            D5 => D6,
            D6 => D7,
            D7 => D8,
            D8 => D9,
            D9 => D0,
        }
    }

    pub fn prev(self) -> Self {
        use PinDigit::*;

        match self {
            D0 => D9,
            D1 => D0,
            D2 => D1,
            D3 => D2,
            D4 => D3,
            D5 => D4,
            D6 => D5,
            D7 => D6,
            D8 => D7,
            D9 => D8,
        }
    }
}
