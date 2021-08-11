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
bottom_color= [0.0, 0.0, 0.0]
top_color = [0.4, 0.0, 0.0]
bar_width = 1.0
buffering = 2
smoothing_size = 3
smoothing_amount = 1
max_frequency = 15000
low_frequency_threshold = 750
low_frequency_scale_doubling = 3
hide_cursor = false

[audio]
pre_fft_windowing = true
";

#[derive(Deserialize, Clone)]
struct Config {
    visual: Visual,
    audio: Audio,
}

#[derive(Deserialize, Clone)]
struct Visual {
    bottom_color: [f32; 3],
    top_color: [f32; 3],
    bar_width: f32,
    buffering: usize,
    smoothing_size: u32,
    smoothing_amount: u32,
    max_frequency: u32,
    low_frequency_threshold: u32,
    low_frequency_scale_doubling: u8,
    hide_cursor: bool,
}

#[derive(Deserialize, Clone)]
struct Audio {
    pre_fft_windowing: bool,
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
    let config: Config = toml::from_str(&config_str).unwrap();

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
        config.visual.bar_width,
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

