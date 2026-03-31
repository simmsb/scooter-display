use buoyant::{
    render::{AnimatedJoin as _, AnimationDomain, Render},
    render_target::{EmbeddedGraphicsRenderTarget, RenderTarget},
};
use embassy_time::{Instant, Timer};

use self::app::App;

#[embassy_executor::task]
pub async fn ui(mut display: crate::display::Display) {
    ui_(display).await;
}

async fn ui_(mut display: crate::display::Display) {
    let mut target =
        EmbeddedGraphicsRenderTarget::new_hinted(&mut display.inner, color::BACKGROUND);

    let app_start = Instant::now();
    let mut app = App::new();

    let mut source_tree = &mut app.tree(target.size().into(), app_start.elapsed().into());
    let mut target_tree = &mut app.tree(target.size().into(), app_start.elapsed().into());

    loop {
        if app.reset_dirty() {
            target_tree.join_from(
                &source_tree,
                &AnimationDomain::top_level(app_start.elapsed().into()),
            );
            core::mem::swap(&mut source_tree, &mut target_tree);
            *target_tree = app.tree(target.size().into(), app_start.elapsed().into());
        }

        if target.clear_animation_status() {
            target.clear(color::BACKGROUND);

            Render::render_animated(
                &mut target,
                source_tree,
                target_tree,
                &color::WHITE,
                &AnimationDomain::top_level(app_start.elapsed().into()),
            );
        } else {
            Timer::after_millis(33).await;
        }
    }
}

mod color {
    use embedded_graphics::prelude::RgbColor;

    pub type ColorFormat = embedded_graphics::pixelcolor::Rgb565;

    pub const GREEN: ColorFormat = ColorFormat::new(20, 200, 50);
    pub const RED: ColorFormat = ColorFormat::new(255, 0, 0);
    pub const YELLOW: ColorFormat = ColorFormat::new(255, 255, 0);
    pub const BLUE: ColorFormat = ColorFormat::new(100, 210, 255);
    pub const BLACK: ColorFormat = ColorFormat::new(0, 0, 0);
    pub const GREY: ColorFormat = ColorFormat::new(150, 150, 150);
    pub const WHITE: ColorFormat = ColorFormat::WHITE;

    pub const BACKGROUND: ColorFormat = WHITE;
    pub const SECONDARY_BACKGROUND: ColorFormat = ColorFormat::new(200, 200, 200);
    pub const CONTENT: ColorFormat = BLACK;
    pub const SECONDARY_CONTENT: ColorFormat = ColorFormat::new(50, 50, 50);
}

mod view {
    use buoyant::{match_view, view::prelude::*};

    use super::color::ColorFormat;

    #[derive(PartialEq, Eq, Clone, Copy, Default)]
    pub enum Screen {
        #[default]
        Homescreen,
        Settings,
    }

    #[must_use]
    pub fn root_view(screen: Screen) -> impl View<ColorFormat, ()> + use<> {
        match_view!(screen, {
            Screen::Homescreen => homescreen::view(),
            Screen::Settings => settings::view(),
        })
        .padding(Edges::All, 5)
    }

    mod homescreen {
        use buoyant::{match_view, view::prelude::*};

        use crate::ui::{
            color::{self, ColorFormat},
            font,
        };

        #[must_use]
        pub fn view() -> impl View<ColorFormat, ()> + use<> {
            ViewThatFits::new(FitAxis::Vertical, {
                VStack::new((
                    labeled_pair("Temperature", "23 C / 73 F", HorizontalAlignment::Leading),
                    labeled_pair("Battery Health", "100 %", HorizontalAlignment::Leading),
                    labeled_pair("Total Input", "12317 wh", HorizontalAlignment::Leading),
                    labeled_pair("Battery Cycles", "142", HorizontalAlignment::Leading),
                    labeled_pair("Total Output", "12247 wh", HorizontalAlignment::Leading),
                    labeled_pair("Screen Uses", "3460", HorizontalAlignment::Leading),
                ))
            })
            .or({
                VStack::new((
                    HStack::new((
                        labeled_pair("Temperature", "23 C / 73 F", HorizontalAlignment::Leading),
                        labeled_pair("Battery Health", "100 %", HorizontalAlignment::Trailing),
                    )),
                    HStack::new((
                        labeled_pair("Total Input", "12317 wh", HorizontalAlignment::Leading),
                        labeled_pair("Battery Cycles", "142", HorizontalAlignment::Trailing),
                    )),
                    HStack::new((
                        labeled_pair("Total Output", "12247 wh", HorizontalAlignment::Leading),
                        labeled_pair("Screen Uses", "3460", HorizontalAlignment::Trailing),
                    )),
                ))
            })
        }

        #[must_use]
        pub fn labeled_pair<'a>(
            label: &'a str,
            value: &'a str,
            alignment: HorizontalAlignment,
        ) -> impl View<ColorFormat, ()> + use<'a> {
            VStack::new((
                Text::new(value, &font::BODY_BOLD).foreground_color(color::CONTENT),
                Text::new(label, &font::FOOTNOTE).foreground_color(color::SECONDARY_CONTENT),
            ))
            .with_alignment(alignment)
            .flex_infinite_width(alignment)
            .with_infinite_max_height()
        }
    }

    mod settings {
        use buoyant::{match_view, view::prelude::*};

        use crate::ui::{color::ColorFormat, font};

        #[must_use]
        pub fn view() -> impl View<ColorFormat, ()> + use<> {
            VStack::new((
                Text::new("Foo", &font::SUBTITLE)
                    .multiline_text_alignment(HorizontalTextAlignment::Center),
                Text::new("Bar", &font::BODY)
                    .multiline_text_alignment(HorizontalTextAlignment::Center),
            ))
            .with_spacing(5)
        }
    }
}

mod app {
    use core::time::Duration;

    use buoyant::view::prelude::*;

    use buoyant::{
        environment::DefaultEnvironment, primitives::ProposedDimensions, render::Render,
    };

    use super::{
        color,
        view::{self, Screen},
    };

    pub struct App {
        state: State,
        is_dirty: bool,
    }

    pub struct State {
        screen: Screen,
    }

    impl App {
        pub fn new() -> Self {
            Self {
                state: State {
                    screen: Screen::Homescreen,
                },
                is_dirty: false,
            }
        }

        pub fn state_mut(&mut self) -> &mut State {
            self.is_dirty = true;
            &mut self.state
        }

        #[must_use]
        pub fn state(&self) -> &State {
            &self.state
        }

        pub fn reset_dirty(&mut self) -> bool {
            let was_dirty = self.is_dirty;
            self.is_dirty = false;
            was_dirty
        }

        pub fn tree(
            &self,
            dimensions: ProposedDimensions,
            app_time: Duration,
        ) -> impl Render<color::ColorFormat> + use<> {
            let env = DefaultEnvironment::new(app_time);
            let view = view::root_view(self.state.screen);
            let mut state = view.build_state(&mut ());
            let layout = view.layout(&dimensions, &env, &mut (), &mut state);
            view.render_tree(
                &layout.sublayouts,
                buoyant::primitives::Point::zero(),
                &env,
                &mut (),
                &mut state,
            )
        }
    }
}

mod font {
    use u8g2_fonts::{
        FontRenderer,
        fonts::{
            u8g2_font_helvB12_tr, u8g2_font_helvB14_tr, u8g2_font_helvB18_tr, u8g2_font_helvR08_tr,
            u8g2_font_helvR12_tr, u8g2_font_helvR18_tr,
        },
    };

    pub static TITLE: FontRenderer = FontRenderer::new::<u8g2_font_helvR18_tr>();
    pub static TITLE_BOLD: FontRenderer = FontRenderer::new::<u8g2_font_helvB18_tr>();
    pub static SUBTITLE: FontRenderer = FontRenderer::new::<u8g2_font_helvB14_tr>();
    pub static BODY: FontRenderer = FontRenderer::new::<u8g2_font_helvR12_tr>();
    pub static BODY_BOLD: FontRenderer = FontRenderer::new::<u8g2_font_helvB12_tr>();
    pub static FOOTNOTE: FontRenderer = FontRenderer::new::<u8g2_font_helvR08_tr>();
}
