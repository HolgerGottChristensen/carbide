#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub fn weight(&self) -> f64 {
        match self {
            FontWeight::W100 | FontWeight::Thin => 100.0,
            FontWeight::W200 | FontWeight::ExtraLight => 200.0,
            FontWeight::W300 | FontWeight::Light => 300.0,
            FontWeight::W400 | FontWeight::Normal => 400.0,
            FontWeight::W500 | FontWeight::Medium => 500.0,
            FontWeight::W600 | FontWeight::SemiBold => 600.0,
            FontWeight::W700 | FontWeight::Bold => 700.0,
            FontWeight::W800 | FontWeight::ExtraBold => 800.0,
            FontWeight::W900 | FontWeight::Black => 900.0,
            FontWeight::Other(val) => *val as f64,
        }
    }
}
