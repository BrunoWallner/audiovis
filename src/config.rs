use std::fs;
use serde::{Deserialize};

const DEFAULT_CONFIG: &str =
"
[visual]
# valid arguments: ['Bars', 'Strings']
visualisation = 'Bars'

bottom_color= [0.0, 0.0, 0.01]
top_color = [1.0, 0.0, 0.0]

# max frequency that should be displayed, lower does not mean any saved work on cpu
max_frequency = 15000
width = 1.0
smoothing_size = 3
smoothing_amount = 1

# hides the cursor if hovered over audiovis window
hide_cursor = false

# start audiovis in fullscreen-mode, pressing F also toggles fullscreen
fullscreen = false
window_always_on_top = false

[processing]
gravity = 0.4

# compensates high and low frequency volume- and space-difference, higher value means less low_frequencies and higher high_frequencies
volume_compensation = 0.6
frequency_compensation = 3.5

[audio]
# should improve quality
pre_fft_windowing = true

volume_amplitude = 0.70
volume_factoring = 0.75
";

#[derive(Deserialize, Clone)]
pub struct Config {
    pub visual: Visual,
    pub processing: Processing,
    pub audio: Audio,
}

#[derive(Deserialize, Clone)]
pub struct Visual {
    pub visualisation: String,
    pub bottom_color: [f32; 3],
    pub top_color: [f32; 3],
    pub smoothing_size: u32,
    pub smoothing_amount: u32,
    pub max_frequency: u32,
    pub width: f32,
    pub hide_cursor: bool,
    pub fullscreen: bool,
    pub window_always_on_top: bool,
}

#[derive(Deserialize, Clone)]
pub struct Audio {
    pub pre_fft_windowing: bool,
    pub volume_amplitude: f32,
    pub volume_factoring: f32,
}

#[derive(Deserialize, Clone)]
pub struct Processing {
    pub gravity: f32,
    pub volume_compensation: f32,
    pub frequency_compensation: f32,
}

pub fn generate_default_config() {
    fs::write("./default_config.toml", DEFAULT_CONFIG).ok();
}

pub fn get_config(path: &str) -> Result<Config, String> {
    // reads config
    let config_str = match path {
        "default" => String::from(DEFAULT_CONFIG),
        _ => {
            match std::fs::read_to_string(path) {
                Ok(config) => config,
                Err(e) => {
                    return Err(format!("could not find config, {}", e));
            }
        }
    }
    };
    let config: Config = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("invalid config: {}", e));
        }
    };
    match check_config(config.clone()) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("invalid config: {}", e));
        }
    }
    Ok(config)
}

pub fn check_config(config: Config) -> Result<(), String> {
    let p = config.processing;
    match config.visual.visualisation.as_str() {
        "Bars" => (),
        "Strings" => (),
        _ => return Err(String::from("error at visual section, invalid visualisation type. Possible types are: 'Bars' and 'Strings'")),
    }
    if p.gravity < 0.0 {
        return Err(String::from("error at processing section, max value for buffering is 100"))
    }
    if config.visual.max_frequency > 20000 || config.visual.max_frequency < 100 {
        return Err(String::from("error at processing section, max_frequency must be in between of 100 and 20.000"))
    }

    Ok(())
}
