use include_dir::{include_dir, Dir};

#[cfg(feature = "lucide")]
static LUCIDE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/lucide");

pub static SYSTEM_IMAGE_MANAGER: fn(&str) -> Option<&'static [u8]> = |name| {

    #[cfg(feature = "lucide")]
    match lucide_icon(name) {
        None => {}
        Some(a) => return Some(a)
    }

    fallback(name)
};

fn fallback(_: &str) -> Option<&'static [u8]> {
    None
}

pub fn all_icon_names() -> Vec<String> {
    let mut icon_names = vec![];

    #[cfg(feature = "lucide")]
    for file in LUCIDE_DIR.find("**/*.svg").unwrap() {
        icon_names.push(file.path().file_stem().unwrap().to_str().unwrap().replace('-', "."));
    }

    icon_names.sort();

    icon_names
}

#[cfg(feature = "lucide")]
fn lucide_icon(name: &str) -> Option<&'static [u8]> {
    let mut name = name.replace('.', "-");
    name.push_str(".svg");

    let file = LUCIDE_DIR.get_file(&name)?;

    Some(file.contents())
}