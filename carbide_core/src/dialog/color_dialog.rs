use crate::environment::Environment;
#[cfg(target_os = "macos")]
use crate::platform::mac::color_dialog::open_color_dialog;
use crate::Color;

pub struct ColorDialog {
    pub(crate) show_alpha: bool,
    pub(crate) continuous: bool,
    //pub(crate) initial_color: Option<Color>,
}

impl ColorDialog {
    pub fn new() -> Self {
        ColorDialog {
            show_alpha: false,
            continuous: true,
            //initial_color: None,
        }
    }

    /*pub fn initial_color(mut self, color: Color) -> Self {
        self.initial_color = Some(color);
        self
    }*/

    pub fn show_alpha(mut self) -> Self {
        self.show_alpha = true;
        self
    }

    pub fn discrete(mut self) -> Self {
        self.continuous = false;
        self
    }

    #[cfg(target_os = "macos")]
    pub fn open(
        self,
        env: &mut Environment,
        color_change: impl Fn(Color, &mut Environment) -> bool + 'static,
    ) {
        let on_next = move |color: Color, env: &mut Environment| -> bool {
            color_change(color, env);
            false
        };

        let sender = env.start_stream(on_next);

        open_color_dialog(env, sender, self);
    }

    #[cfg(not(target_os = "macos"))]
    pub fn open(
        mut self,
        env: &mut Environment,
        color_change: impl Fn(Color, &mut Environment) -> bool + 'static,
    ) {
        todo!()
    }
}
