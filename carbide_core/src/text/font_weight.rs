#[derive(Copy, Clone, Debug)]
pub enum FontWeight {
    Bold,
    // Normally w700
    Normal,
    // Normally w400
    W100,
    W200,
    W300,
    W400,
    W500,
    W600,
    W700,
    W800,
    W900,
}

impl FontWeight {
    pub fn weight(&self) -> f64 {
        match self {
            FontWeight::Bold => 700.0,
            FontWeight::Normal => 400.0,
            FontWeight::W100 => 100.0,
            FontWeight::W200 => 200.0,
            FontWeight::W300 => 300.0,
            FontWeight::W400 => 400.0,
            FontWeight::W500 => 500.0,
            FontWeight::W600 => 600.0,
            FontWeight::W700 => 700.0,
            FontWeight::W800 => 800.0,
            FontWeight::W900 => 900.0,
        }
    }
}