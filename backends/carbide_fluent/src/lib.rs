use std::{fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use fluent::FluentResource;
use icu::locid::{Locale};
use lazy_static::lazy_static;
use walkdir::WalkDir;

use carbide_core::{impl_read_state, locate_folder};
pub use localized_string::*;
pub use localized_datetime::*;
pub use localized_number::*;
pub use localizable::Localizable;
pub use args::Arg;
pub use args::LocalizedArg;
pub use locale_ext::LocaleExt;
pub use icu::locid::locale;

mod localized_string;
mod localizable;
mod args;
mod locale_ext;
mod localized_datetime;
mod localized_number;


type Bundle = fluent::bundle::FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>;

lazy_static!(
    static ref LANGUAGES: HashMap<Locale, Bundle> = {
        load_languages().unwrap()
    };
);

fn load_languages() -> Result<HashMap<Locale, Bundle>, io::Error> {
    let assets = locate_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();

    let dir = fs::read_dir(assets.join("i18n"))?;
    let mut locales = HashMap::new();

    for entry in dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name) = name.to_str() {
                        let langid = Locale::from_str(name).expect("Parsing failed.");
                        let mut bundle = fluent::bundle::FluentBundle::new_concurrent(vec![langid.clone()]);

                        for entry in WalkDir::new(path) {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                if path.is_file() {
                                    let mut f = File::open(path)?;
                                    let mut s = String::new();
                                    f.read_to_string(&mut s)?;

                                    let resource = FluentResource::try_new(s).expect("Could not parse an FTL string.");

                                    bundle.add_resource(resource);
                                }
                            }
                        }

                        locales.insert(langid, bundle);
                    }
                }
            }
        }
    }

    Ok(locales)
}
