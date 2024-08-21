use std::{
    sync::mpsc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use log::info;

use crate::{
    activity_strings::ActivityStrings, connection_plot::ConnectionPlot,
    data_collection::RustyBridgeData, latency_plot::LatencyPlot,
};

pub trait Display {
    fn is_enabled(&self, _ctx: &egui::Context) -> bool {
        true
    }

    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, data: &RustyBridgeData);
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    data: RustyBridgeData,
    last_data_update_ms: wasm_timer::Instant,
    refresh_time_ms: u64,
    activity_strings: ActivityStrings,
    latency_plot: LatencyPlot,
    connection_plot: ConnectionPlot,
}

//impl Default for TemplateApp {
//fn default() -> Self {
//Self {
//activity_strings: ActivityStrings::default(),
//latency_plot: LatencyPlot::default(),
//connection_plot: ConnectionPlot::default(),
//last_data_update_ms: wasm_timer::Instant::now(),
//refresh_time_ms: 5000,
//data: RustyBridgeData::default(),
//}
//}
//}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        let data = RustyBridgeData::new();

        TemplateApp {
            data,
            last_data_update_ms: wasm_timer::Instant::now(),
            refresh_time_ms: 5000,
            activity_strings: ActivityStrings::default(),
            latency_plot: LatencyPlot::default(),
            connection_plot: ConnectionPlot::default(),
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    //fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //eframe::set_value(storage, eframe::APP_KEY, self);
    //}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // Update data if needed
        update_data(self);

        // popup window
        //self.activity_strings.show(ctx, &mut true);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label("bottom panel test");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rusty-bridge metrics viewer");
            ui.label("All kinds of metrics data will be presented here");
            ui.label("connection events");
            ui.label("message history");
            ui.add_space(20.0);
            ui.vertical(|ui| {
                self.latency_plot.ui(ui, &self.data);
                ui.add_space(30.0);
                ui.code_editor(&mut format!("{:?}", self.data.connection_events));
            });
        });

        egui::SidePanel::right("activity_panel")
            .resizable(false)
            .exact_width(300.0)
            .show(ctx, |ui| {
                ui.label("Data update activity");
                self.activity_strings.ui(ui, &self.data);
                ui.add(
                    egui::Slider::new(&mut self.refresh_time_ms, 50..=10_000)
                        .text("Update interval"),
                );
                //ui.text_edit_singleline(&mut self.refresh_time_ms);
                ui.add_space(300.0);
                //ui.text_edit_multiline(&mut "connection events goes here".to_string());
                ui.label("todo");
                self.connection_plot.ui(ui, &self.data);
            });
    }
}

fn update_data(app: &mut TemplateApp) {
    // Always check for data that has been returned from a request
    app.data.try_recv();

    // Send out a new request if the elapsed time has passed (can't loop an async routine in wasm)
    let now = app.last_data_update_ms.elapsed();
    let time_since_update = (app.last_data_update_ms - now).elapsed().as_millis();
    let time_since_update = time_since_update as u64;
    // Update every `refresh_time_ms` interval
    if time_since_update > app.refresh_time_ms {
        app.data.get_msg_events();
        app.data.get_conn_events();
        app.last_data_update_ms = wasm_timer::Instant::now();
    }
}

fn get_time_now() -> u128 {
    0
    //Instant::now().elapsed().as_millis()
    //SystemTime::now()
    //.duration_since(UNIX_EPOCH)
    //.unwrap_or(Duration::new(u64::MAX, u32::MAX))
    //.as_millis()
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
