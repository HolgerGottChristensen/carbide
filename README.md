# Carbide

Carbide is a rust GUI Framework in its experimental stages. It is build to be simple to use and simple to layout
widgets. It is inspired by current attempts at creating desktop UI, such as SwiftUI, Flutter and Qt.

Carbide is a fork of the earlier repository conrod (which is still being worked on and maintained).

Carbide is an attempt to create an easy-to-use, 2D library written entirely in Rust.

The project differentiates itself from other attempts (druid, egui, iced, ...) by using the mindset of SwiftUI layouting
along with being a retained mode framework.

## Goals

My goal is to be able to use this for small to medium applications. It is written completely in rust and the goal is you
don't have to leave the rust ecosystem to be able to develop quality applications.

* **Simple layouts** -- by implementing the pattern common from SwiftUI. This means we do not have more complex layouts
  for example from flexbox. This is of course implementable using a custom layout widget, but will probably not be
  provided.

* **Simple to implement custom widgets** -- by using rust proc macros to implement a lot of the boilerplate for you.
  This should mean you will not have to implement a lot of functions where the default behavior is fitting.

* **Renderer and window independent** -- by creating a clear interface between the framework, and the underneath layers
  such as window handling and rendering.

* **Platform agnostic** -- by providing a common interface to multiple platforms. There is currently not any plans for
  running the framework on mobile devices, but experiments have been made.

* **Accessibility** -- by integrating with up-incoming rust accessibility crates.

* **Localization** -- by providing widgets that are aware of language and providing an asset format for translations. We
  are looking at rust attempts at font shaping and ligatures, but it seems to be in its early stages.

## Non-goals

This is not an immediate mode framework and is not trying to be (look at egui, imgui or iced). We are not trying to copy
the native looking widgets... (Todo)

## Current state

Currently, the framework is in the early stages, but most of the layouting algorithms and basic widgets have been
implemented. By extending conrod, I am trying to bring the project up to more modern standards, 2018 rust, with its new
features.

## Feedback

If you try the framework, feel free to open an issue, or provide any feedback on how difficult it is to use. Currently,
I am the only developer of this framework, and mostly spending my spare time contributing to this.

License
-------

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

**Example Assets**

- [Google Noto](https://www.google.com/get/noto/) (Apache2)

[The API Documentation]: https://docs.rs/carbide_core/

[The Guide]: https://docs.rs/carbide_core/latest/carbide_core/guide/index.html

[1]:        https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html

[1.1]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#a-brief-history

[1.2]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#screenshots-and-videos

[1.3]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#feature-overview

[1.4]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#available-widgets

[1.4.1]:    https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#primitive-widgets

[1.4.2]:    https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#common-use-widgets

[1.5]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#immediate-mode

[1.5.1]:    https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#what-is-it

[1.5.2]:    https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#why-use-it

[1.5.3]:    https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#is-carbide-immediate-or-retained

[1.6]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_1/index.html#the-builder-pattern

[2]:        https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_2/index.html

[2.1]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_2/index.html#installing-rust-and-cargo

[2.2]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_2/index.html#running-the-carbide-examples

[3]:        https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_3/index.html

[3.1]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_3/index.html#creating-a-new-project

[3.2]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_3/index.html#setting-up-carbide

[3.2.1]:    https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_3/index.html#backends

[3.3]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_3/index.html#creating-a-window

[3.4]:      https://docs.rs/carbide_core/latest/carbide_core/guide/chapter_3/index.html#handling-events

[issues]: https://github.com/PistonDevelopers/carbide/issues

[1.0.0 milestone]: https://github.com/PistonDevelopers/carbide/milestones/1.0.0

[Contributing]: https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md
