use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Window,
};

use wgpu::util::DeviceExt;

use std::sync::mpsc;
use std::thread;

mod bridge;

mod graphics;
use graphics::*;

mod audio;

use serde::{Deserialize};

const DEFAULT_CONFIG: &str =
"
[visual]
visualisation = 'Bars'

bottom_color= [0.0, 0.0, 0.0]
top_color = [0.2, 0.0, 0.0]
bar_width = 1.0
buffering = 2
smoothing_size = 4
smoothing_amount = 1
max_frequency = 15000

low_frequency_threshold = 100
low_frequency_scale_doubling = 3
low_frequency_smoothing_size = 8
low_frequency_smoothing = 1

low_frequency_fading = 2.0
low_frequency_volume_reduction = 2.0

hide_cursor = false

[audio]
pre_fft_windowing = true
volume_amplitude = 1.0
";

#[derive(Deserialize, Clone)]
struct Config {
    visual: Visual,
    audio: Audio,
}

#[derive(Deserialize, Clone)]
struct Visual {
    visualisation: String,
    bottom_color: [f32; 3],
    top_color: [f32; 3],
    width: f32,
    buffering: usize,
    smoothing_size: u32,
    smoothing_amount: u32,
    max_frequency: u32,

    low_frequency_threshold: u32,
    low_frequency_scale_doubling: u8,
    low_frequency_smoothing: u8,
    low_frequency_smoothing_size: u32,
    low_frequency_fading: f32,
    low_frequency_volume_reduction: f32,

    hide_cursor: bool,
}

#[derive(Deserialize, Clone)]
struct Audio {
    pre_fft_windowing: bool,
    volume_amplitude: f32,
}

fn main() {
    // reads config
    let config_str = match std::fs::read_to_string("config.toml") {
        Ok(config) => config,
        Err(e) => {
            println!("could not find config.toml: {}, falling back to default config", e);
            DEFAULT_CONFIG.to_string()
        }
    };
    let config: Config = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            println!("invalid config: {}", e);
            std::process::exit(1);
        }
    };
    match check_config(config.clone()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("invalid config: {}", e);
            std::process::exit(1);
        }
    }


    // initiates communication bridge between audio input and wgpu
    let (bridge_sender, bridge_receiver) = mpsc::channel();
    let sender_clone = bridge_sender.clone();
    bridge::init(
        bridge_receiver,
        sender_clone,
        config.visual.buffering,
        config.visual.smoothing_size,
        config.visual.smoothing_amount,
        config.visual.max_frequency,
        config.visual.low_frequency_threshold,
        config.visual.low_frequency_scale_doubling,
        config.visual.low_frequency_volume_reduction,
        config.visual.low_frequency_smoothing,
        config.visual.low_frequency_smoothing_size,
        config.visual.low_frequency_fading,
    );
    let config_clone = config.clone();
    let sender_clone = bridge_sender.clone();
    thread::spawn(move|| {
        audio::init(
            sender_clone,
            config_clone.visual.max_frequency,
            config_clone.audio.pre_fft_windowing,
        )
    });

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(String::from("audiovis"))
        .build(&event_loop).unwrap();

    // window configuration
    window.set_cursor_visible(!config.visual.hide_cursor);

    let mut state = pollster::block_on(State::new(
        &window,
        bridge_sender.clone(),
        config.visual.top_color,
        config.visual.bottom_color,
        config.visual.width,
        config.audio.volume_amplitude,
        config.visual.visualisation,
    ));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) { // UPDATED!
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn check_config(config: Config) -> Result<(), String> {
    let v = config.visual;
    match v.visualisation.as_str() {
        "Bars" => (),
        "Strings" => (),
        _ => return Err(String::from("error at visual section, invalid visualisation type. Possible types are: 'Bars' and 'Strings'")),
    }
    if v.buffering > 100 {
        return Err(String::from("error at visual section, max value for buffering is 100"))
    }
    if v.max_frequency > 20000 || v.max_frequency < 100 {
        return Err(String::from("error at visual section, max_frequency must be in between of 100 and 20.000"))
    }
    if v.low_frequency_threshold > v.max_frequency / 2 {
        return Err(String::from("error at visual section, low_frequency_threshold must be lower than half of max_frequency"))
    }

    Ok(())
}

