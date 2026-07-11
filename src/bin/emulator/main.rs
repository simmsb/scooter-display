use std::time::Instant;

use buoyant::{
    app::{App, Harness as _},
    event::Event,
    focus::Role,
    render_target::{EmbeddedGraphicsRenderTarget, RenderTarget as _},
    view::View,
};
use eframe::egui;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay};
use scooter_display::{
    cfg::SpeedMode,
    operation::OperationCommand,
    pin_digit::PinDigit,
    sim::{self, SimState},
    ui::{
        self, colour,
        engine::UiEngine,
        keys,
        state::{Page, State},
    },
};

const DISPLAY_W: u32 = 320;
const DISPLAY_H: u32 = 480;
const SCALE: f32 = 2.0;

fn speed_mode_label(mode: SpeedMode) -> &'static str {
    match mode {
        SpeedMode::Walk => "Walk",
        SpeedMode::Eco => "Eco",
        SpeedMode::Trip => "Trip",
        SpeedMode::Sport => "Sport",
    }
}

const SPEED_MODES: [SpeedMode; 4] = [
    SpeedMode::Walk,
    SpeedMode::Eco,
    SpeedMode::Trip,
    SpeedMode::Sport,
];

fn main() -> eframe::Result {
    let mut display = SimulatorDisplay::new(Size::new(DISPLAY_W, DISPLAY_H));
    let target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display, colour::black());
    let size = target.size();

    let mut app = App::new(ui::state::State::new(), size, ui::view::root_view)
        .with_roles(Role::Button | Role::Container);
    app.focus_forward();

    let emulator = EmulatorApp::new(app, display);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([DISPLAY_W as f32 * SCALE + 320.0, DISPLAY_H as f32 * SCALE]),
        ..Default::default()
    };

    eframe::run_native(
        "Scooter Display Emulator",
        options,
        Box::new(|_| Ok(Box::new(emulator))),
    )
}

struct EmulatorApp<V, F>
where
    V: View<colour::ColorFormat, State>,
    F: Fn(&State) -> V,
{
    sim: SimState,
    display: SimulatorDisplay<Rgb565>,
    app: App<V, State, F>,
    ui_engine: UiEngine,
    app_start: Instant,
    texture: Option<egui::TextureHandle>,
    pending_events: Vec<Event>,
    speed_kmh: f32,
    throttle: f32,
    battery_soc: f32,
    battery_voltage: f32,
    battery_current: f32,
    battery_temp: f32,
    ambient_light: f32,
    odometer: f32,
    predicted_range: f32,
    locked: bool,
    pin_digits: [u8; 4],
    can_alive: bool,
    headlight_on: bool,
    brake_light_on: bool,
    charging: bool,
    charged: bool,
    speed_mode_idx: usize,
    speed_limit: u8,
    theme_settings: colour::ThemeSettings,
}

impl<V, F> EmulatorApp<V, F>
where
    V: View<colour::ColorFormat, State>,
    F: Fn(&State) -> V,
{
    fn new(app: App<V, State, F>, display: SimulatorDisplay<Rgb565>) -> Self {
        let sim = SimState::new();
        let speed_kmh = sim.system.motor_speed as f32 / 100.0;
        let throttle = sim.system.throttle.0 as f32;
        let battery_soc = sim.system.battery_info.relative_soc as f32;
        let battery_voltage = sim.system.system_voltage.from_battery as f32 / 1000.0;
        let battery_current = sim.system.battery_current as f32 / 1000.0;
        let battery_temp = sim.system.battery_info.temperature as f32;
        let ambient_light = sim.system.ambient_light.mapped as f32;
        let odometer = sim.system.odometer as f32;
        let predicted_range = sim.system.predicted_range as f32;
        let locked = sim.operation.is_locked();
        let pin_digits = [2, 7, 0, 8];
        let can_alive = sim::can_is_alive();
        let headlight_on = sim.system.headlight_on;
        let brake_light_on = sim.system.brake_light_on;
        let charging = sim.system.battery_info.charging;
        let charged = sim.system.battery_info.charged;
        let speed_limit = 22;

        Self {
            sim,
            display,
            app,
            ui_engine: UiEngine::new(),
            app_start: Instant::now(),
            texture: None,
            pending_events: Vec::new(),
            speed_kmh,
            throttle,
            battery_soc,
            battery_voltage,
            battery_current,
            battery_temp,
            ambient_light,
            odometer,
            predicted_range,
            locked,
            pin_digits,
            can_alive,
            headlight_on,
            brake_light_on,
            charging,
            charged,
            speed_mode_idx: 0,
            speed_limit,
            theme_settings: colour::theme_settings(),
        }
    }

    fn send_click(&mut self, key: u8) {
        self.pending_events.push(Event::KeyDown(key));
        self.pending_events.push(Event::KeyUp(key));
    }

    fn send_hold(&mut self, key: u8) {
        self.pending_events.push(Event::KeyDown(key));
        self.pending_events.push(Event::KeyUp(key));
    }

    fn apply_sim_inputs(&mut self) {
        self.sim.system.motor_speed = (self.speed_kmh * 100.0) as u16;
        self.sim.system.throttle.0 = self.throttle as u16;
        self.sim.system.battery_info.relative_soc = self.battery_soc as u8;
        self.sim.system.battery_info.level_from_controller = self.battery_soc as u8;
        self.sim.system.battery_info.absolute_soc = (self.battery_soc * 50.0) as u16;
        self.sim.system.system_voltage.from_battery = (self.battery_voltage * 1000.0) as u16;
        self.sim.system.system_voltage.from_controller =
            (self.battery_voltage * 1000.0 + 1200.0) as u16;
        self.sim.system.battery_current = (self.battery_current * 1000.0) as i16;
        self.sim.system.battery_info.temperature = self.battery_temp as i16;
        self.sim.system.ambient_light.mapped = self.ambient_light as u8;
        self.sim.system.odometer = self.odometer as u16;
        self.sim.system.predicted_range = self.predicted_range as u16;
        self.sim.system.headlight_on = self.headlight_on;
        self.sim.system.brake_light_on = self.brake_light_on;
        self.sim.system.battery_info.charging = self.charging;
        self.sim.system.battery_info.charged = self.charged;

        sim::set_can_alive(self.can_alive);

        let digits = [
            PinDigit::from_char(char::from_digit(self.pin_digits[0] as u32, 10).unwrap()).unwrap(),
            PinDigit::from_char(char::from_digit(self.pin_digits[1] as u32, 10).unwrap()).unwrap(),
            PinDigit::from_char(char::from_digit(self.pin_digits[2] as u32, 10).unwrap()).unwrap(),
            PinDigit::from_char(char::from_digit(self.pin_digits[3] as u32, 10).unwrap()).unwrap(),
        ];
        self.sim.set_unlock_code_digits(digits);
        self.sim.set_locked(self.locked);

        if let Some(active) = self.sim.operation.as_active() {
            self.speed_mode_idx = SPEED_MODES
                .iter()
                .position(|m| *m == active.speed_mode)
                .unwrap_or(0);
            self.speed_limit = active.speed_limit;
        }

        if self.sim.needs_ui_sync(self.app.state()) {
            self.sim.sync_to_ui(&mut *self.app.state_mut());
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let output = self.display.to_rgb_output_image(&OutputSettings::default());
        let image = egui::ColorImage::from_rgb(
            [DISPLAY_W as usize, DISPLAY_H as usize],
            output.as_image_buffer().as_raw(),
        );

        match &mut self.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => {
                self.texture =
                    Some(ctx.load_texture("display", image, egui::TextureOptions::NEAREST));
            }
        }
    }

    fn apply_theme_settings(&mut self, settings: colour::ThemeSettings) {
        if colour::set_theme_settings(settings) {
            self.theme_settings = settings;
            self.app.force_rebuild();
        }
    }
}

impl<V, F> eframe::App for EmulatorApp<V, F>
where
    V: View<colour::ColorFormat, State>,
    F: Fn(&State) -> V,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_sim_inputs();

        let events = std::mem::take(&mut self.pending_events);
        let mut target =
            EmbeddedGraphicsRenderTarget::new_hinted(&mut self.display, colour::black());
        let tick =
            self.ui_engine
                .tick(&mut self.app, &mut target, self.app_start.elapsed(), events);

        self.sim
            .apply_operation_commands(tick.operation_commands.into_iter());

        self.locked = self.sim.operation.is_locked();

        if tick.rendered {
            self.update_texture(ctx);
        }

        egui::SidePanel::right("controls")
            .default_width(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Theme");
                    let mut hsva = egui::ecolor::Hsva::new(
                        self.theme_settings.hue / 360.0,
                        self.theme_settings.saturation,
                        self.theme_settings.value,
                        1.0,
                    );
                    if egui::color_picker::color_picker_hsva_2d(
                        ui,
                        &mut hsva,
                        egui::color_picker::Alpha::Opaque,
                    ) {
                        self.apply_theme_settings(colour::ThemeSettings {
                            hue: hsva.h * 360.0,
                            saturation: hsva.s,
                            value: hsva.v,
                            dark_mode: self.theme_settings.dark_mode,
                        });
                    }
                    if ui
                        .button(if self.theme_settings.dark_mode {
                            "Switch to light mode"
                        } else {
                            "Switch to dark mode"
                        })
                        .clicked()
                    {
                        self.apply_theme_settings(colour::ThemeSettings {
                            dark_mode: !self.theme_settings.dark_mode,
                            ..self.theme_settings
                        });
                    }

                    ui.separator();
                    ui.heading("Buttons");
                    ui.horizontal(|ui| {
                        if ui.button("Up click").clicked() {
                            self.send_click(keys::UP_CLICK);
                        }
                        if ui.button("Up hold").clicked() {
                            self.send_hold(keys::UP_HOLD);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Down click").clicked() {
                            self.send_click(keys::DOWN_CLICK);
                        }
                        if ui.button("Down hold").clicked() {
                            self.send_hold(keys::DOWN_HOLD);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Confirm click").clicked() {
                            self.send_click(keys::CONFIRM_CLICK);
                        }
                        if ui.button("Confirm hold").clicked() {
                            self.send_hold(keys::CONFIRM_HOLD);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Power click").clicked() {
                            self.send_click(keys::POWER_CLICK);
                        }
                        if ui.button("Power hold").clicked() {
                            self.send_hold(keys::POWER_HOLD);
                        }
                    });

                    ui.separator();
                    ui.heading("Motion");
                    ui.add(egui::Slider::new(&mut self.speed_kmh, 0.0..=45.0).text("Speed (km/h)"));
                    ui.add(egui::Slider::new(&mut self.throttle, 0.0..=450.0).text("Throttle"));

                    ui.separator();
                    ui.heading("Battery");
                    ui.add(egui::Slider::new(&mut self.battery_soc, 0.0..=100.0).text("SOC (%)"));
                    ui.add(
                        egui::Slider::new(&mut self.battery_voltage, 30.0..=54.0)
                            .text("Voltage (V)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.battery_current, -20.0..=10.0)
                            .text("Current (A)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.battery_temp, -10.0..=60.0)
                            .text("Temperature (°C)"),
                    );
                    ui.checkbox(&mut self.charging, "Charging");
                    ui.checkbox(&mut self.charged, "Charged");

                    ui.separator();
                    ui.heading("System");
                    ui.checkbox(&mut self.headlight_on, "Headlight on");
                    ui.checkbox(&mut self.brake_light_on, "Brake light on");
                    ui.add(
                        egui::Slider::new(&mut self.ambient_light, 0.0..=64.0)
                            .text("Ambient light"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.odometer, 0.0..=9999.0).text("Odometer (km)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.predicted_range, 0.0..=200.0)
                            .text("Predicted range (km)"),
                    );
                    ui.checkbox(&mut self.can_alive, "CAN alive");

                    ui.separator();
                    ui.heading("Lock");
                    ui.checkbox(&mut self.locked, "Locked");
                    ui.horizontal(|ui| {
                        for (i, digit) in self.pin_digits.iter_mut().enumerate() {
                            ui.add(
                                egui::DragValue::new(digit)
                                    .range(0..=9)
                                    .prefix(format!("PIN {i}: ")),
                            );
                        }
                    });

                    ui.separator();
                    ui.heading("Operation");
                    ui.label("Speed mode");
                    let previous_speed_mode_idx = self.speed_mode_idx;
                    let unlocked = !self.sim.operation.is_locked();
                    ui.horizontal_wrapped(|ui| {
                        for (i, mode) in SPEED_MODES.iter().enumerate() {
                            ui.add_enabled_ui(unlocked, |ui| {
                                ui.radio_value(
                                    &mut self.speed_mode_idx,
                                    i,
                                    speed_mode_label(*mode),
                                );
                            });
                        }
                    });
                    if unlocked && self.speed_mode_idx != previous_speed_mode_idx {
                        self.sim
                            .apply_operation_commands([OperationCommand::SetSpeedMode(
                                SPEED_MODES[self.speed_mode_idx],
                            )]);
                    }
                    let previous_speed_limit = self.speed_limit;
                    ui.add_enabled_ui(unlocked, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.speed_limit)
                                .range(0..=45)
                                .prefix("Speed limit: "),
                        );
                    });
                    if unlocked && self.speed_limit != previous_speed_limit {
                        self.sim
                            .apply_operation_commands([OperationCommand::SetSpeedLimit(
                                self.speed_limit,
                            )]);
                    }
                    let speed_limit_unlocked = self
                        .sim
                        .operation
                        .as_active()
                        .map(|a| a.speed_limit_unlocked)
                        .unwrap_or(false);
                    if ui
                        .add_enabled(
                            unlocked && !speed_limit_unlocked,
                            egui::Button::new("Unlock speed limit"),
                        )
                        .clicked()
                    {
                        self.sim
                            .apply_operation_commands([OperationCommand::UnlockSpeedLimit]);
                    }

                    ui.separator();
                    ui.heading("Debug");
                    ui.label(format!(
                        "Page: {}",
                        match self.app.state().page {
                            Page::Home => "Home",
                            Page::Settings => "Settings",
                            Page::Info => "Info",
                        }
                    ));
                    if ui.button("Force redraw").clicked() {
                        ctx.request_repaint();
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                ui.add(
                    egui::Image::new((
                        texture.id(),
                        egui::vec2(DISPLAY_W as f32 * SCALE, DISPLAY_H as f32 * SCALE),
                    ))
                    .fit_to_exact_size(egui::vec2(
                        DISPLAY_W as f32 * SCALE,
                        DISPLAY_H as f32 * SCALE,
                    )),
                );
            } else {
                ui.label("Waiting for first frame…");
            }
        });

        if tick.rendered {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}
