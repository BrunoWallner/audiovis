use std::fs;
use serde::{Deserialize};

const DEFAULT_CONFIG: &str =
"
[visual]
# valid arguments: ['Bars', 'Strings']
visualisation = 'Bars'

bottom_color= [0.0, 0.0, 0.0]
top_color = [1.0, 0.0, 0.0]
max_frequency = 15000
width = 1.0
smoothing_size = 3
smoothing_amount = 1
hide_cursor = false

# pressing F also toggles fullscreen
fullscreen = false
window_always_on_top = false

[processing]
buffering = 2
low_frequency_threshold = 50
low_frequency_scale_doubling = 6
low_frequency_smoothing_size = 3
low_frequency_smoothing = 1

# higher value means less low_frequencies and higher high_frequencies
volume_compensation = 0.55

# the further away from 1.0 the more fading, this could distort frequency threshold
low_frequency_fading = 1.75
low_frequency_volume_reduction = true

[audio]
pre_fft_windowing = true
volume_amplitude = 0.75
volume_factoring = 0.65
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
    pub buffering: usize,
    pub low_frequency_threshold: u32,
    pub low_frequency_scale_doubling: u8,
    pub low_frequency_smoothing: u8,
    pub low_frequency_smoothing_size: u32,
    pub low_frequency_fading: f32,
    pub volume_compensation: f32,
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
    if p.buffering > 100 {
        return Err(String::from("error at processing section, max value for buffering is 100"))
    }
    if config.visual.max_frequency > 20000 || config.visual.max_frequency < 100 {
        return Err(String::from("error at processing section, max_frequency must be in between of 100 and 20.000"))
    }
    if p.low_frequency_threshold > config.visual.max_frequency / 2 {
        return Err(String::from("error at processing section, low_frequency_threshold must be lower than half of max_frequency"))
    }

    Ok(())
}
