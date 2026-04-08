use buoyant::{
    app::Harness as _,
    event::Event,
    focus::Role,
    render::{AnimatedJoin as _, AnimationDomain, Render},
    render_target::{EmbeddedGraphicsRenderTarget, RenderTarget}, view::ViewLayout,
};
use embassy_time::{Duration, Instant, Ticker, Timer};

use self::state::{Page, PageAction, State};

#[embassy_executor::task]
pub async fn ui(mut display: crate::display::Display) {
    ui_(display).await;
}


const fn root_view_differ_size<V, T, S>(f: fn(T) -> V) -> usize
where
    V: ViewLayout<S>,
    V::Renderables: buoyant::render::Diffable,
{
    use buoyant::render::Diffable;

    V::Renderables::SIZE.div_ceil(8) + 1
}

async fn ui_(mut display: crate::display::Display) {
    let mut target =
        EmbeddedGraphicsRenderTarget::new_hinted(&mut display.inner, color::BACKGROUND);

    let app_start = Instant::now();

    let mut app = buoyant::app::App::new(
        State {
            foo: 0,
            page: Page::Homescreen,
            page_action: None,
        },
        target.size().into(),
        view::root_view,
    )
    .with_roles(Role::Button | Role::Container);

    app.focus_forward();

    let mut last_changed = Instant::now();
    let mut last_changed_foo = Instant::now();

    target.clear(color::BACKGROUND);

    let mut diffing_mem = [0u8; root_view_differ_size(view::root_view)];

    loop {
        app.set_time(app_start.elapsed().into());

        // Todo: events
        if last_changed.elapsed() > Duration::from_secs(5) {
            last_changed = Instant::now();

            app.send(Event::KeyDown(buoyant::event::Key::UpArrow));
        }

        if last_changed_foo.elapsed() > Duration::from_secs(1) {
            last_changed_foo = Instant::now();

            app.state_mut().foo += 1;
        }

        // Handle page changes
        if let Some(action) = app.state().page_action {
            let current_page = app.state().page;
            let new_page = current_page.handle_action(action);
            let mut state = app.state_mut();
            state.page = new_page;
            state.page_action = None;

            defmt::trace!("Page: {}", state.page);
        }

        if app.should_redraw() || target.clear_animation_status() {
            defmt::trace!("Redrawing");

            app.render_animated_diffed(&mut target, &color::BACKGROUND, &mut diffing_mem);

            // debug focus?
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
    use core::time::Duration;

    use buoyant::{
        event::{Event, Key},
        focus::{self, FocusAction},
        match_view,
        render::Capsule,
        view::prelude::*,
    };

    use crate::ui::{
        color,
        state::{Page, PageAction},
    };

    use super::{color::ColorFormat, state::State};

    #[must_use]
    pub fn root_view(state: &State) -> impl View<ColorFormat, State> + use<> {
        let paginate = move |s: &mut State, a: buoyant::view::paginate::PageEvent| {
            s.page_action = Some(match a {
                buoyant::view::paginate::PageEvent::Next => PageAction::Next,
                buoyant::view::paginate::PageEvent::Previous => PageAction::Prev,
            });
        };

        buoyant::view::Paginate::new(focus::GROUP_1, paginate, {
            match_view!(state.page, {
                Page::Homescreen => homescreen::view()
                    .bound_focus(focus::BoundaryBehavior::Wrap),
                Page::Settings => settings::view(state)
                    .bound_focus(focus::BoundaryBehavior::Wrap),
            })
        })
        .map_event::<(), _>(|event: &Event, _state| match event {
            Event::KeyDown(key) => match key {
                Key::UpArrow => Some(FocusAction::Previous.into_event(focus::GROUP_1)),
                Key::DownArrow => Some(FocusAction::Next.into_event(focus::GROUP_1)),
                _ => None,
            },
            _ => Some(event.clone()),
        })
        .padding(Edges::All, 5)
        .background_color(color::BACKGROUND, Rectangle)
    }

    mod homescreen {
        use buoyant::{match_view, view::prelude::*};

        use crate::ui::{
            color::{self, ColorFormat},
            font,
            state::State,
        };

        #[must_use]
        pub fn view() -> impl View<ColorFormat, State> + use<> {
            VStack::new((
                labeled_pair("Temperature", "23 C / 73 F", HorizontalAlignment::Leading),
                labeled_pair("Battery Health", "100 %", HorizontalAlignment::Leading),
                labeled_pair("Total Input", "12317 wh", HorizontalAlignment::Leading),
                labeled_pair("Battery Cycles", "142", HorizontalAlignment::Leading),
                labeled_pair("Total Output", "12247 wh", HorizontalAlignment::Leading),
                labeled_pair("Screen Uses", "3460", HorizontalAlignment::Leading),
            ))
        }

        #[must_use]
        pub fn labeled_pair<'a, S>(
            label: &'a str,
            value: &'a str,
            alignment: HorizontalAlignment,
        ) -> impl View<ColorFormat, S> + use<'a, S> {
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

        use crate::ui::{
            color::{self, ColorFormat},
            font,
            state::State,
        };

        #[must_use]
        pub fn view(state: &State) -> impl View<ColorFormat, State> + use<> {
            VStack::new((
                Text::new("Foo", &font::TITLE)
                    .multiline_text_alignment(HorizontalTextAlignment::Center)
                    .foreground_color(color::CONTENT),
                Text::new(heapless::format!(8; "{}", state.foo).unwrap(), &font::BODY)
                    .multiline_text_alignment(HorizontalTextAlignment::Center)
                    .foreground_color(color::SECONDARY_CONTENT),
            ))
            .with_alignment(HorizontalAlignment::Center)
            .flex_infinite_width(HorizontalAlignment::Center)
            .with_infinite_max_height()
        }
    }
}

mod state {
    #[derive(PartialEq, Eq, Clone, Copy, Default, defmt::Format)]
    pub enum Page {
        #[default]
        Homescreen,
        Settings,
    }

    impl Page {
        pub fn handle_action(&self, action: PageAction) -> Self {
            match (self, action) {
                (Page::Homescreen, PageAction::Next) => Page::Settings,
                (Page::Homescreen, PageAction::Prev) => Page::Settings,

                (Page::Settings, PageAction::Next) => Page::Homescreen,
                (Page::Settings, PageAction::Prev) => Page::Homescreen,
            }
        }
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum PageAction {
        Next,
        Prev,
    }

    pub struct State {
        pub foo: u32,
        pub page: Page,
        pub page_action: Option<PageAction>,
    }
}

mod font {
    use u8g2_fonts::{
        FontRenderer,
        fonts,
    };

    pub static TITLE: FontRenderer = FontRenderer::new::<fonts::u8g2_font_eckpixel_tr>();
    pub static TITLE_BOLD: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvB18_tr>();
    pub static SUBTITLE: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvB14_tr>();
    pub static BODY: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvR12_tr>();
    pub static BODY_BOLD: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvB12_tr>();
    pub static FOOTNOTE: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvR08_tr>();
}
