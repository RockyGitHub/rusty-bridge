use std::f64::consts::TAU;

use egui::{remap, Color32, NumExt, Response, Ui};

use crate::{
    data_collection::RustyBridgeData,
    egui_plot::{CoordinatesFormatter, Corner, Legend, Line, LineStyle, Plot, PlotPoints},
};

#[derive(Default)]
pub struct ConnectionPlot {
    reserved: u32,
    time: f64,
}

impl ConnectionPlot {
    pub fn ui(&mut self, ui: &mut Ui, data: &RustyBridgeData) -> Response {
        //self.options_ui(ui);

        //ui.ctx().request_repaint();
        self.time += ui.input(|i| i.unstable_dt).at_most(1.0 / 30.0) as f64;
        let mut plot = Plot::new("Connectivity")
            .legend(Legend::default())
            .y_axis_width(4)
            //.height(300.0)
            //.width(600.0)
            .x_axis_label("Time (_)")
            .y_axis_label("Connected (1/0)")
            .show_axes(true)
            .show_grid(true);
        plot = plot.view_aspect(1.0);
        plot = plot.data_aspect(1.0);
        plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        plot.show(ui, |plot_ui| {
            plot_ui.line(self.circle());
            plot_ui.line(self.sin());
            plot_ui.line(self.thingy());
        })
        .response
    }
    fn circle(&self) -> Line {
        let n = 512;
        let circle_points: PlotPoints = (0..=n)
            .map(|i| {
                let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
                let r = 5.0;
                [r * t.cos() + 0.0 as f64, r * t.sin() + 0.0 as f64]
            })
            .collect();
        Line::new(circle_points)
            .color(Color32::from_rgb(100, 200, 100))
            .style(LineStyle::Solid)
            .name("circle")
    }

    fn sin(&self) -> Line {
        let time = self.time;
        Line::new(PlotPoints::from_explicit_callback(
            move |x| 0.5 * (2.0 * x).sin() * time.sin(),
            ..,
            512,
        ))
        .color(Color32::from_rgb(200, 100, 100))
        .style(LineStyle::Solid)
        .name("wave")
    }

    fn thingy(&self) -> Line {
        let time = self.time;
        Line::new(PlotPoints::from_parametric_callback(
            move |t| ((2.0 * t + time).sin(), (3.0 * t).sin()),
            0.0..=TAU,
            256,
        ))
        .color(Color32::from_rgb(100, 150, 250))
        .style(LineStyle::Solid)
        .name("x = sin(2t), y = sin(3t)")
    }
}
