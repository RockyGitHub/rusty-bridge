//mod bar_chart;
//mod bar;

use egui::{Color32, Id, Response, Ui};
use log::info;

use crate::{data_collection::{msg_events::MsgEventData, RustyBridgeData}, egui_plot::{Bar, BarChart, Legend, Plot}};

pub struct LatencyPlot {
    allow_zoom: bool,
    allow_drag: bool,
    allow_scroll: bool,
    vertical: bool,
    chart: BarChart,
}

impl Default for LatencyPlot {
    fn default() -> Self {
        Self {
            chart: BarChart::new(Vec::with_capacity(200)),
            allow_zoom: true,
            allow_drag: true,
            allow_scroll: true,
            vertical: true,
        }
    }
}

impl LatencyPlot {
    pub fn ui(&mut self, ui: &mut Ui, data: &RustyBridgeData) -> Response {
        // bars example
        //let bars = (-395..=395)
            //.step_by(10)
            //.map(|x| x as f64 * 0.01)
            //.map(|x| {
                //(
                    //x,
                    //(-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt(),
                //)
            //})
            //// The 10 factor here is purely for a nice 1:1 aspect ratio
            //.map(|(x, f)| Bar::new(x, f * 10.0).width(0.095))
            //.collect();


        let mut bars = Vec::new();
        for event in &data.msg_events {
            // math..   split the events into the bars. the higher the bar, the more events fell into that time range
            let bar = match event.ack_time_ms {
                Some(ack_time) => Bar::new(event.id as f64, (ack_time - event.publish_time_ms) as f64).width(0.85),
                None => Bar::new(event.id as f64, 100.0).fill(Color32::LIGHT_RED).width(0.895),
            };
            bars.push(bar);
        }

        let mut chart = BarChart::new(bars)
            .color(Color32::LIGHT_BLUE)
            .name("Message RTT (round trip time)");

        if !self.vertical {
            chart = chart.horizontal();
        }
        Plot::new("Normal Distribution Demo")
            .legend(Legend::default())
            .clamp_grid(false)
            .y_axis_width(3)
            .allow_zoom(self.allow_zoom)
            .allow_drag(self.allow_drag)
            .allow_scroll(self.allow_scroll)
            .height(400.0)
            .width(800.0)
            .x_axis_label("Msg id")
            .y_axis_label("Latency (ms)")
            .show(ui, |plot_ui| plot_ui.bar_chart(chart))
            .response
    }
}
