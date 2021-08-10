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

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Config {
    visual: Visual
}

#[derive(Deserialize)]
struct Visual {
    bottom_color: [f32; 3],
    top_color: [f32; 3],
    bars: u32,
    bar_width: f32,
    buffering: usize,
    smoothing_size: u32,
    smoothing_amount: u32,
    interpolate: bool,
}




fn main() {
    // reads config
    let config_str = match std::fs::read_to_string("config.toml") {
        Ok(config) => config,
        Err(e) => {
            println!("could not find config.toml:\n{}", e);
            std::process::exit(1);
        }
    };
    let mut config: Config = toml::from_str(&config_str).unwrap();
    // config check
    if config.visual.smoothing_size > config.visual.bars {
        println!("[ERROR]: Invalid config, smoothing_size cant be greater than amount of bars");
        config.visual.smoothing_size = config.visual.bars;
    }

    // initiates communication bridge between audio input and wgpu
    let (bridge_sender, bridge_receiver) = mpsc::channel();
    bridge::init(bridge_receiver, config.visual.buffering, config.visual.smoothing_size, config.visual.smoothing_amount, config.visual.bars, config.visual.interpolate);
    let sender_clone = bridge_sender.clone();
    thread::spawn(move|| {
        audio::init(sender_clone)
    });

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(
        &window,
        bridge_sender.clone(),
        config.visual.top_color,
        config.visual.bottom_color,
        config.visual.bars,
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

