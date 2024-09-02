use maud::{html, Markup, PreEscaped, Render};

#[derive(Clone, Copy, Debug)]
pub struct Icon(&'static str);

impl Icon {
    pub fn inline(self) -> Markup {
        html! {
            span .inline-icon { (self) }
        }
    }

    fn svg(&self) -> &str {
        // Remove any trailing newlines from SVG file contents
        self.0.trim_end()
    }
}

impl Render for Icon {
    fn render(&self) -> Markup {
        html! {
            (PreEscaped(self.svg()))
        }
    }
}

macro_rules! icon {
    ($var:ident, $name:literal) => {
        pub const $var: Icon =
            Icon(include_str!(concat!("icon/", $name, ".svg")));
    };
}

icon!(ALIGN_LEFT, "align_left");
icon!(ARROW_DOWN_A_Z, "arrow-down-a-z");
icon!(ARROW_DOWN_Z_A, "arrow-down-z-a");
icon!(ARROW_DOWN_NARROW_WIDE, "arrow-down-narrow-wide");
icon!(ARROW_DOWN_WIDE_NARROW, "arrow-down-wide-narrow");
icon!(BADGE_CHECK, "badge_check");
icon!(CALENDAR, "calendar");
icon!(CIRCLE_USER_ROUND, "circle-user-round");
icon!(CLOCK, "clock");
icon!(CLOCK_ARROW_DOWN, "clock-arrow-down");
icon!(CLOCK_ARROW_UP, "clock-arrow-up");
icon!(COMMENT, "comment");
icon!(DOTS_6_ROTATE, "6-dots-rotate");
icon!(ENVELOPE, "envelope");
icon!(EYE, "eye");
icon!(FILE, "file");
icon!(FILE_FILL, "file_fill");
icon!(FILE_IMAGE, "file_image");
icon!(HASH, "hash");
icon!(HOME, "home");
icon!(LINK, "link");
icon!(LOG_IN, "log-in");
icon!(LOG_OUT, "log-out");
icon!(MAGNIFYING_GLASS, "magnifying_glass");
icon!(PENCIL, "pencil");
icon!(PLUS, "plus");
icon!(ROTATE_CW, "rotate-cw");
icon!(SETTINGS, "settings");
icon!(SQUARE_PEN, "square-pen");
icon!(USER_CIRCLE, "user_circle");
icon!(X, "x");
