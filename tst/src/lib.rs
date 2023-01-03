use chrono::prelude::*;
use eframe::egui::Ui;
use egui_datepicker::DatePicker;
use std::fmt::Display;

// struct App<Tz>
// where
//     Tz: TimeZone,
//     Tz::Offset: Display,
// {
//     date: chrono::Date<Tz>,
// }
// impl<Tz> App<Tz>
// where
//     Tz: TimeZone,
//     Tz::Offset: Display,
// {
//     fn draw_datepicker(&mut self, ui: &mut Ui) {
//         ui.add(DatePicker::new("super_unique_id", &mut self.date));
//     }
// }