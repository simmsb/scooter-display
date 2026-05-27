#[rustfmt::skip]
#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default, derive_enum_rotate::EnumRotate)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum PinDigit {
    #[default]
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
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
}
