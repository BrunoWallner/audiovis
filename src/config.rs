use std::fs;
use serde::{Deserialize};

const DEFAULT_CONFIG: &str =
"
[visual]
# path to bar_texture, 'default' for default texture
texture = 'default'

camera_pos = [0.0, -0.25, 17.5]
camera_facing = [-0.05, -0.25, 0.0]
fov = 5

# max frequency that should be displayed, lower does not mean any saved work on cpu
max_frequency = 20000
width = 1.0
z_width = 0.5

smoothing_size = 10
smoothing_amount = 3

# hides the cursor if hovered over audiovis window
hide_cursor = false

# start audiovis in fullscreen-mode, pressing F also toggles fullscreen
fullscreen = false
window_always_on_top = false

[processing]
gravity = 1.0

# higher resolution adds latency and processing time
resolution = 2048

# normalizes the position of bars, higher value encreases proportions of lower frequencies
# default value should be 0.5 but with favourite_freq_scaling this should be increased
normalisation_factoring = 0.5

# range of frequencies which scale should be increased
fav_frequency_range = [40, 3500]
fav_frequency_doubling = 2

# how many buffers should be saved and displayed in 3D
buffering = 25

# halfes the scale x times
bar_reduction = 2

buffer_resolution_drop = 1.0
max_buffer_resolution_drop = 8

[audio]
# should improve quality
pre_fft_windowing = true

volume_amplitude = 100.0
volume_factoring = 1.0
";

#[derive(Deserialize, Clone)]
pub struct Config {
    pub visual: Visual,
    pub processing: Processing,
    pub audio: Audio,
}

#[derive(Deserialize, Clone)]
pub struct Visual {
    pub smoothing_size: u32,
    pub smoothing_amount: u32,
    pub max_frequency: u32,
    pub width: f32,
    pub z_width: f32,
    pub hide_cursor: bool,
    pub fullscreen: bool,
    pub window_always_on_top: bool,
    pub camera_pos: [f32; 3],
    pub camera_facing: [f32; 3],
    pub fov: f32,
    pub texture: String,
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
    pub normalisation_factoring: f32,
    pub fav_frequency_range: [u32; 2],
    pub fav_frequency_doubling: u16,
    pub buffering: u32,
    pub bar_reduction: u32,
    pub buffer_resolution_drop: f32,
    pub max_buffer_resolution_drop: u16,
    pub resolution: u32,
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
    if p.gravity < 0.0 {
        return Err(String::from("error at processing section, max value for buffering is 100"))
    }
    if config.visual.max_frequency > 20000 || config.visual.max_frequency < 100 {
        return Err(String::from("error at processing section, max_frequency must be in between of 100 and 20.000"))
    }
    if p.gravity < 0.0 {
        return Err(String::from("error at processing section, gravity must be greater than 0.0"))
    }

    Ok(())
}
