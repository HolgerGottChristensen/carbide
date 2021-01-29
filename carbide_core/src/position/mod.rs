//! Items related to 2D positioning, used throughout carbide.

pub use self::range::{Edge, Range};
pub use self::rect::{Corner, Rect};
pub mod range;
pub mod rect;


/// An alias over the Scalar type used throughout carbide.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;

/// The depth at which the widget will be rendered.
///
/// This determines the order of rendering where widgets with a greater depth will be rendered
/// first.
///
/// 0.0 is the default depth.
pub type Depth = f32;

/// General use 2D spatial dimensions.
pub type Dimensions = [Scalar; 2];

/// General use 2D spatial point.
pub type Point = [Scalar; 2];

/// The margin for some `Place`ment on either end of an axis.
pub type Margin = Scalar;

/// Represents either **Axis** in the 2-dimensional plane.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Axis {
    /// The horizontal plane's Axis.
    X,
    /// The vertical plane's Axis.
    Y,
}

/// Some **Position** of some **Widget** along a single axis.
///
/// **Position**s for both the *x* and *y* axes are stored internally within the
/// **widget::CommonBuilder** type, allowing all widgets to be positioned in a variety of different
/// ways.
///
/// See the [**Positionable**](./trait.Positionable) trait for methods that allow for setting the
/// **Position**s in various ways.
///
/// Note that **Positionable** is implemented for *all* render that implement **Widget**.

/// Positions that are described as **Relative** to some other **Widget**.
///
/// **Relative** describes a relative position along a single axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Relative {
    /// A relative scalar distance.
    Scalar(Scalar),
    /// Aligned to either the `Start`, `Middle` or `End`.
    Align(Align),
    /// A distance as a `Scalar` value over the given `Direction`.
    Direction(Direction, Scalar),
    /// Some place on top of another widget.
    ///
    /// Similar to `Align`, but represents the `Start`/`End` of the other widget's `kid_area`.
    ///
    /// Also allows for specifying a `Margin` from either end.
    ///
    /// Using `Place` allows the `Ui` to infer the widget's parent as the widget upon which it is
    /// `Placed`, though this inferrence only occurs if the `parent` was not specifically set.
    Place(Place),
}

/// Directionally positioned, normally relative to some other widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Positioned forwards (*positive* **Scalar**) along some **Axis**.
    Forwards,
    /// Positioned backwards (*negative* **Scalar**) along some **Axis**.
    Backwards,
}

/// The orientation of **Align**ment along some **Axis**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Align {
    /// **Align** our **Start** with the **Start** of some other widget along the **Axis**.
    Start,
    /// **Align** our **Middle** with the **Middle** of some other widget along the **Axis**.
    Middle,
    /// **Align** our **End** with the **End** of some other widget along the **Axis**.
    End,
}

/// Place the widget at a position on some other widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Place {
    /// Place upon the **Start** of the Widget's `kid_area`.
    Start(Option<Margin>),
    /// Place upon the **Middle** of the Widget's `kid_area`.
    Middle,
    /// Place upon the **End** of the Widget's `kid_area`.
    End(Option<Margin>),
}