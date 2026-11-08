mod common;

use crate::common::build_application;

fn main() {
    let mut application = build_application("Remix");

    application.launch()
}
