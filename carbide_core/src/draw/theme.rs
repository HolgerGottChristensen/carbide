use crate::environment::EnvironmentColor;
use crate::environment::EnvironmentVariable;
use carbide_core::color;

macro_rules! env_color {
    ($var:ident, $r:literal, $g:literal, $b:literal, $a:literal) => {
        EnvironmentVariable::Color {
            key: EnvironmentColor::$var,
            value: color::rgba_bytes($r, $g, $b, $a),
        }
    };
}

pub fn light_mode_color_theme() -> Vec<EnvironmentVariable> {
    vec![
        EnvironmentVariable::Color {
            key: EnvironmentColor::Blue,
            value: color::rgba_bytes(0, 122, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Green,
            value: color::rgba_bytes(52, 199, 89, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Indigo,
            value: color::rgba_bytes(88, 86, 214, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Orange,
            value: color::rgba_bytes(255, 149, 0, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Pink,
            value: color::rgba_bytes(255, 45, 85, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Purple,
            value: color::rgba_bytes(175, 82, 222, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Red,
            value: color::rgba_bytes(255, 59, 48, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Teal,
            value: color::rgba_bytes(90, 200, 250, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Yellow,
            value: color::rgba_bytes(255, 204, 0, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray,
            value: color::rgba_bytes(142, 142, 147, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray2,
            value: color::rgba_bytes(174, 174, 178, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray3,
            value: color::rgba_bytes(199, 199, 204, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray4,
            value: color::rgba_bytes(209, 209, 214, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray5,
            value: color::rgba_bytes(229, 229, 234, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray6,
            value: color::rgba_bytes(242, 242, 247, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SystemBackground,
            value: color::rgba_bytes(255, 255, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SecondarySystemBackground,
            value: color::rgba_bytes(242, 242, 247, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::TertiarySystemBackground,
            value: color::rgba_bytes(255, 255, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Label,
            value: color::rgba_bytes(0, 0, 0, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SecondaryLabel,
            value: color::rgba_bytes(138, 138, 142, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::TertiaryLabel,
            value: color::rgba_bytes(196, 196, 198, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::QuaternaryLabel,
            value: color::rgba_bytes(220, 220, 221, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::PlaceholderText,
            value: color::rgba_bytes(196, 196, 198, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Link,
            value: color::rgba_bytes(0, 122, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SystemFill,
            value: color::rgba_bytes(228, 228, 230, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SecondarySystemFill,
            value: color::rgba_bytes(233, 233, 235, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::TertiarySystemFill,
            value: color::rgba_bytes(239, 239, 240, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::QuaternarySystemFill,
            value: color::rgba_bytes(244, 244, 245, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::OpaqueSeparator,
            value: color::rgba_bytes(220, 220, 222, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Separator,
            value: color::rgba_bytes(0, 0, 0, 0.137),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Accent,
            value: color::rgba_bytes(0, 122, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::LightText,
            value: color::rgba_bytes(0, 0, 0, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::DarkText,
            value: color::rgba_bytes(255, 255, 255, 1.0),
        },
        // Material colors
        env_color!(UltraThick, 255, 255, 255, 0.8),
        env_color!(Thick, 255, 255, 255, 0.6),
        env_color!(Regular, 255, 255, 255, 0.4),
        env_color!(Thin, 255, 255, 255, 0.25),
        env_color!(UltraThin, 255, 255, 255, 0.15),
        // Material colors light
        env_color!(UltraThickLight, 255, 255, 255, 0.8),
        env_color!(ThickLight, 255, 255, 255, 0.6),
        env_color!(RegularLight, 255, 255, 255, 0.4),
        env_color!(ThinLight, 255, 255, 255, 0.25),
        env_color!(UltraThinLight, 255, 255, 255, 0.15),
        // Material colors dark
        env_color!(UltraThickDark, 0, 0, 0, 0.8),
        env_color!(ThickDark, 0, 0, 0, 0.6),
        env_color!(RegularDark, 0, 0, 0, 0.4),
        env_color!(ThinDark, 0, 0, 0, 0.25),
        env_color!(UltraThinDark, 0, 0, 0, 0.15),
    ]
}

pub fn dark_mode_color_theme() -> Vec<EnvironmentVariable> {
    vec![
        EnvironmentVariable::Color {
            key: EnvironmentColor::Blue,
            value: color::rgba_bytes(10, 132, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Green,
            value: color::rgba_bytes(48, 209, 88, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Indigo,
            value: color::rgba_bytes(94, 92, 230, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Orange,
            value: color::rgba_bytes(255, 149, 10, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Pink,
            value: color::rgba_bytes(255, 55, 95, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Purple,
            value: color::rgba_bytes(191, 90, 242, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Red,
            value: color::rgba_bytes(255, 69, 58, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Teal,
            value: color::rgba_bytes(100, 210, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Yellow,
            value: color::rgba_bytes(255, 214, 10, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray,
            value: color::rgba_bytes(142, 142, 147, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray2,
            value: color::rgba_bytes(99, 99, 102, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray3,
            value: color::rgba_bytes(72, 72, 74, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray4,
            value: color::rgba_bytes(58, 58, 60, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray5,
            value: color::rgba_bytes(44, 44, 46, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Gray6,
            value: color::rgba_bytes(28, 28, 30, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SystemBackground,
            value: color::rgba_bytes(28, 28, 30, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SecondarySystemBackground,
            value: color::rgba_bytes(44, 44, 46, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::TertiarySystemBackground,
            value: color::rgba_bytes(58, 58, 60, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Label,
            value: color::rgba_bytes(255, 255, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SecondaryLabel,
            value: color::rgba_bytes(152, 152, 159, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::TertiaryLabel,
            value: color::rgba_bytes(90, 90, 95, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::QuaternaryLabel,
            value: color::rgba_bytes(65, 65, 69, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::PlaceholderText,
            value: color::rgba_bytes(71, 71, 74, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Link,
            value: color::rgba_bytes(9, 132, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SystemFill,
            value: color::rgba_bytes(61, 61, 65, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::SecondarySystemFill,
            value: color::rgba_bytes(57, 57, 61, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::TertiarySystemFill,
            value: color::rgba_bytes(50, 50, 54, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::QuaternarySystemFill,
            value: color::rgba_bytes(44, 44, 48, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::OpaqueSeparator,
            value: color::rgba_bytes(61, 61, 65, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Separator,
            value: color::rgba_bytes(255, 255, 255, 0.15),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::Accent,
            value: color::rgba_bytes(10, 132, 255, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::LightText,
            value: color::rgba_bytes(0, 0, 0, 1.0),
        },
        EnvironmentVariable::Color {
            key: EnvironmentColor::DarkText,
            value: color::rgba_bytes(255, 255, 255, 1.0),
        },
        // Material colors
        env_color!(UltraThick, 0, 0, 0, 0.8),
        env_color!(Thick, 0, 0, 0, 0.6),
        env_color!(Regular, 0, 0, 0, 0.4),
        env_color!(Thin, 0, 0, 0, 0.25),
        env_color!(UltraThin, 0, 0, 0, 0.15),
        // Material colors light
        env_color!(UltraThickLight, 255, 255, 255, 0.8),
        env_color!(ThickLight, 255, 255, 255, 0.6),
        env_color!(RegularLight, 255, 255, 255, 0.4),
        env_color!(ThinLight, 255, 255, 255, 0.25),
        env_color!(UltraThinLight, 255, 255, 255, 0.15),
        // Material colors dark
        env_color!(UltraThickDark, 0, 0, 0, 0.8),
        env_color!(ThickDark, 0, 0, 0, 0.6),
        env_color!(RegularDark, 0, 0, 0, 0.4),
        env_color!(ThinDark, 0, 0, 0, 0.25),
        env_color!(UltraThinDark, 0, 0, 0, 0.15),
    ]
}
