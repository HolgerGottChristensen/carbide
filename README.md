# Carbide
[![crates.io](https://img.shields.io/crates/v/carbide)](https://crates.io/crates/carbide)
[![docs.rs](https://docs.rs/carbide_core/badge.svg)](https://docs.rs/carbide_core/)
[![license](https://img.shields.io/crates/l/carbide)]()

Carbide is an attempt to create an easy-to-use, 2D library written entirely in Rust (through it is possible to integrate with rend3 for 3d ).

Carbide is an experimental Rust GUI Framework. The main focuses of the Carbide project are simplicity and composability. It is inspired by current attempts at creating desktop UI, such as SwiftUI, Flutter and Qt.

Carbide is a fork of the earlier repository called conrod (which is no longer maintained). Since the fork, almost all the code has been rewritten, so the two
are not really comparable anymore.

The project differentiates itself from other attempts (druid, egui, iced, ...) by using the mindset of SwiftUI layouting
along with being a retained mode framework.

## Examples
All the screenshots are taken on macOS, but they have also been tested on Windows. 
### A simple counter
Clicking the button will increase the number. 
Notice we are able to create the text directly from an integer state. This will become useful in the future.
You will also notice we have to add a font manually. In the future we will have the default font of the OS loaded
and this will be a way to add custom fonts.

The button has hover and press effects and this is customizable.
You might also notice we specify the font size from the environment. This means we choose
a semantic size instead of an absolute size. Font sizes will be able to change
based on accessibility requirements. The color of the button is blue because that is the 
default accent color.
![Counter application](https://user-images.githubusercontent.com/11473146/156854780-ae51c267-ed5b-4f73-9999-521edf763e53.png)

```rust
use carbide::{Application, Window, a};
use carbide::draw::Dimension;
use carbide::state::LocalState;
use carbide::widget::{Text, VStack, WidgetExt};
use carbide::controls::{Button};
use carbide::environment::EnvironmentFontSize::LargeTitle;

fn main() {
  let mut application = Application::new()
          .with_asset_fonts();

  let counter = LocalState::new(0);

  let text = Text::new(counter.clone()).font_size(LargeTitle);

  let button = Button::new_primary("Increase counter", a!(|_, _| {
        *$counter += 1;
    }))
          .frame(200.0, 30.0);

  application.set_scene(Window::new(
    "My first counter",
    Dimension::new(300.0, 235.0),
    VStack::new((
      text,
      button
    ))
  ).close_application_on_window_close());

  application.launch()
}
```

### Materials
We have a couple of materials that can be used to give different feels of depth to your application.
We have materials from UltraThick on the left all the way to UltraThin on the right. The top row is light theme
and the bottom row shows the materials in dark theme.
![A window that shows different materials applied to rectangles on colored backgrounds](https://user-images.githubusercontent.com/11473146/156855641-c94b1e73-2222-4370-ab58-3a3ccaf20b25.png "Materials window") 
Code for the example can be found at: [Materials Example]

### Shapes
Carbide has a few different shapes built in like rectangles, rounded rectangles, capsules, ellipsis and more. 
They also support filling, stroking, both and to be used as clip masks for other items such as images.
The bottom row of stars is build using a canvas element similar to Canvas2d from html.
![This image shows the different supported shapes in carbide. It also showes an example use of the canvas element where you can create your own shapes.](https://user-images.githubusercontent.com/11473146/156856089-31fa94a5-e951-42f9-8828-8140c1f24cf0.png "Shapes")
Code for the example can be found at: [Shapes Example]

### Simple hacker-news client
I have started building a small hacker news client. Currently, it supports fetching messages asynchronously 
using rust async and Carbide's built in task! macro. We have support for async-std and Tokio, and more could easily be added.

It has a selectable list which is build in to Carbide, which supports both single select and multi-selection.
When clicking on an article it shows the title and the link for the article. Comments are not currently fetched,
because I from their firebase api was not able to extract them without sending hundreds of requests.
![](https://user-images.githubusercontent.com/11473146/156856663-1621d9f2-f4cb-4494-902e-eab0fbb68cd8.png "A simple hacker-news client")

### Calculator
A simple calculator. The logic of the calculator might not be the fanciest, but the UI defined in 
Carbide should work quite nicely. 
![](https://user-images.githubusercontent.com/11473146/156857158-3431f365-c3b5-4659-956a-3bef1abac5cc.png "A simple calculator")
Code for the example can be found at: [Calculator Example]

### Text edits
A short GIF showing the text edits. The first two edits are using the same state, and because of that
if you edit one, it will also sync the other. The last three is also connected to the same state.

The two first text fields are connected to i128 state. This means that it validated that the current 
string in the text field is convertable to i128, otherwise it will show an error. This means validation
that it is not empty, it does not contain letters and other symbols, and that the number is within range of
what an i128 can contain. This is all out of the box, just by giving an i128 state. The same will work for
f64 and all other number types in rust. Currently, there is no support for localizations, but will be added soon.

The last two text fields are obscured which means you are not able to see or get the data out of the field.
This is useful for typing passwords. You are able to choose which character should be used as the obscuring character.

Text fields support most common shortcuts like copy, paste, select all, select word, and more except undo/redo. It also 
supports mouse interactions like double click for select word, triple click for select all and click and drag for selecting
a section of the content.

![Carbide text input example](https://user-images.githubusercontent.com/11473146/156861242-399869ae-42b5-4f06-9045-0f9ce40cacfb.gif)

![Carbide text input example 2](https://user-images.githubusercontent.com/11473146/156861582-d4c2c76d-d881-4f77-8827-90266b79d4c0.gif)

Code for the example can be found at: [Textfield Example]

## Documentation
I have started documenting the main parts of the projects and are planning on starting a mdbook 
but currently this is in the early stages. 

The easiest way to get started is by looking at the examples in the carbide_wgpu, carbide_controls and carbide crates. These examples show a wide variety of projects and functionalities.
While this is not the ideal form of documentation, it should get you started.

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

* **Accessibility** -- by integrating with up-incoming rust accessibility crates. [AccessKit](https://github.com/AccessKit/accesskit) looks promising, but progress is slow.

* **Localization** -- by providing widgets that are aware of language and providing an asset format for translations. We
  are looking at rust attempts at font shaping and ligatures, but it seems to be in its early stages. Currently I am considering 
  [Fluent](https://github.com/projectfluent/fluent-rs) for specifying translations. [https://github.com/kellpossible/cargo-i18n](https://github.com/kellpossible/cargo-i18n)
  also looks promising.

## Non-goals

This is not an immediate mode framework and is not trying to be (look at egui, imgui or iced). We are not trying to copy
the native looking widgets... (Todo)

## Current state

Currently, the framework is in the early stages, but most of the layouting algorithms and basic widgets have been
implemented. By extending Conrod, I am trying to bring the project up to more modern standards, 2018 rust, with its new
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

[Materials Example]: https://github.com/HolgerGottChristensen/carbide/blob/master/backends/carbide_wgpu/examples/materials.rs
[Shapes Example]: https://github.com/HolgerGottChristensen/carbide/blob/master/backends/carbide_wgpu/examples/shapes.rs
[Calculator Example]: https://github.com/HolgerGottChristensen/carbide/tree/master/carbide/examples/calculator
[Textfield Example]: https://github.com/HolgerGottChristensen/carbide/blob/master/carbide_controls/examples/text_input.rs
[The API Documentation]: https://docs.rs/carbide_core/

[issues]: https://github.com/HolgerGottChristensen/carbide/issues

[1.0.0 milestone]: https://github.com/HolgerGottChristensen/carbide/milestones/1.0.0

[Contributing]: https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md
