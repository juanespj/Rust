extern crate touch_visualizer;
use touch_visualizer::TouchVisualizer;

pub use crate::gui::Tube;

pub fn ui_keyboard( args: &Key) {
    
    println!("Pressed keyboard key '{:?}'", key);
    if key == Key::Left{
        tube_a.alph+=PI/10.0;
        if tube_a.alph > 2.0*PI{
            tube_a.alph =0.0;
        }
    }
    if key == Key::Right{
        tube_a.alph-=PI/10.0;
        if tube_a.alph <0.0 {
            tube_a.alph =2.0*PI;
        }
    }

}