use carbide_derive::StateValue;

#[derive(Copy, Clone, Debug, PartialEq, StateValue)]
pub enum FontWeight {
    Thin,
    W100,

    ExtraLight,
    W200,

    Light,
    W300,

    Normal,
    W400,

    Medium,
    W500,

    SemiBold,
    W600,

    Bold,
    W700,

    ExtraBold,
    W800,

    Black,
    W900,

    Other(u16),
}

impl FontWeight {
    pub fn weight(&self) -> u16 {
        match self {
            FontWeight::W100 | FontWeight::Thin => 100,
            FontWeight::W200 | FontWeight::ExtraLight => 200,
            FontWeight::W300 | FontWeight::Light => 300,
            FontWeight::W400 | FontWeight::Normal => 400,
            FontWeight::W500 | FontWeight::Medium => 500,
            FontWeight::W600 | FontWeight::SemiBold => 600,
            FontWeight::W700 | FontWeight::Bold => 700,
            FontWeight::W800 | FontWeight::ExtraBold => 800,
            FontWeight::W900 | FontWeight::Black => 900,
            FontWeight::Other(val) => *val,
        }
    }
}
