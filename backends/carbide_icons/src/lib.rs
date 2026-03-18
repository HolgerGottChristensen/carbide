use include_dir::{include_dir, Dir};

#[cfg(feature = "lucide")]
static LUCIDE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/lucide");

#[cfg(feature = "tabler")]
static TABLER_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/tabler");

#[cfg(feature = "remix")]
static REMIX_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/remix");

pub static SYSTEM_IMAGE_MANAGER: fn(&str) -> Option<&'static [u8]> = |name| {

    #[cfg(feature = "lucide")]
    match lucide_icon(name) {
        None => {}
        Some(a) => return Some(a)
    }

    #[cfg(feature = "tabler")]
    match tabler_icon(name) {
        None => {}
        Some(a) => return Some(a)
    }

    #[cfg(feature = "remix")]
    match remix_icon(name) {
        None => {}
        Some(a) => return Some(a)
    }

    None
};

pub fn all_icon_names() -> Vec<String> {
    let mut icon_names = vec![];

    #[cfg(feature = "lucide")]
    icon_names.append(&mut lucide_icon_names());

    #[cfg(feature = "tabler")]
    icon_names.append(&mut tabler_icon_names());

    #[cfg(feature = "remix")]
    icon_names.append(&mut remix_icon_names());

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

#[cfg(feature = "lucide")]
fn lucide_icon_names() -> Vec<String> {
    let mut names = vec![];
    for file in LUCIDE_DIR.find("**/*.svg").unwrap() {
        names.push(file.path().file_stem().unwrap().to_str().unwrap().replace('-', "."));
    }
    names
}

#[cfg(feature = "tabler")]
fn tabler_icon(name: &str) -> Option<&'static [u8]> {
    let filled = name.ends_with(".fill");

    let mut name = if filled {
        format!("filled/{}", &name[..(name.len()-5)])
    } else {
        format!("outline/{}", name)
    };

    name = name.replace('.', "-");
    name.push_str(".svg");

    let file = TABLER_DIR.get_file(&name)?;

    Some(file.contents())
}

#[cfg(feature = "tabler")]
fn tabler_icon_names() -> Vec<String> {
    let mut names = vec![];
    for file in TABLER_DIR.find("filled/**/*.svg").unwrap() {
        let mut name = file.path().file_stem().unwrap().to_str().unwrap().replace('-', ".");
        name.push_str(".fill");
        names.push(name);
    }

    for file in TABLER_DIR.find("outline/**/*.svg").unwrap() {
        let mut name = file.path().file_stem().unwrap().to_str().unwrap().replace('-', ".");
        names.push(name);
    }

    names
}

#[cfg(feature = "remix")]
fn remix_icon(name: &str) -> Option<&'static [u8]> {
    let filled = name.ends_with(".fill");

    let name = name.replace('.', "-");

    let name2 = if filled {
        format!("**/{}.svg", name)
    } else {
        format!("**/{}-line.svg", name)
    };

    let name = &format!("**/{}.svg", name);

    let entry = REMIX_DIR.find(name)
        .unwrap()
        .next()
        .or_else(|| {
            REMIX_DIR.find(&name2)
                .unwrap()
                .next()
        })?;

    let file = REMIX_DIR.get_file(entry.path())?;

    Some(file.contents())
}

#[cfg(feature = "remix")]
fn remix_icon_names() -> Vec<String> {
    let mut names = vec![];
    for file in REMIX_DIR.find("**/*.svg").unwrap() {
        let mut name = file.path().file_stem().unwrap().to_str().unwrap().replace('-', ".");

        if name.ends_with(".line") {
            name = name[..(name.len()-5)].to_string();
        }

        names.push(name);
    }

    names
}