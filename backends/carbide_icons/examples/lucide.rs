mod common;

use crate::common::build_application;

fn main() {
    let application = build_application("Lucide");

    application.launch()
}
