#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod activity_strings;
mod connection_plot;
mod latency_plot;
mod egui_plot;
mod data_collection;
mod error;

pub use app::TemplateApp;
pub use app::Display;
pub use app::View;
pub use error::Error;