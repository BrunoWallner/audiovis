use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Window,
    window::Fullscreen,
};
use wgpu::util::DeviceExt;
use std::sync::mpsc;

mod bridge;

mod config;
mod audio;
mod graphics;
use graphics::wgpu_abstraction::State;

use clap::{Arg, App};

fn main() {
    //env_logger::init();

    let matches = App::new("audiovis")
        .version("0.1.0")
        .author("Luca Biendl <b.lucab1211@gmail.com>")
        .about("tool to visualize audio")
        .arg(Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .takes_value(true)
                    .help("use custom configuration"))

        .arg(Arg::with_name("input_device")
                    .short("i")
                    .long("input-device")
                    .takes_value(false)
                    .help("use input device to visualize"))

        .arg(Arg::with_name("generate_default_config")
                    .short("g")
                    .long("generate-default-config")
                    .takes_value(false)
                    .help("generates default configuration"))

        .get_matches();

    let config_path: &str = matches.value_of("config").unwrap_or("default");
    let mut audio_device: audio::DeviceType = audio::DeviceType::Output();
    if matches.is_present("input_device") {
        audio_device = audio::DeviceType::Input();
    }

    if matches.is_present("generate_default_config") {
        config::generate_default_config();
        println!("generated default config");
        std::process::exit(0);
    }

    let config = match config::get_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let default_bar_texture = include_bytes!("default_bar_texture.png");
    let bar_texture = match config.visual.texture.as_str() {
        "default" => {default_bar_texture.to_vec()},
        other => {
            match std::fs::read(other) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("could not load bar_texture.png {}, falling back to default texture", e);
                    default_bar_texture.to_vec()
                }
            }
        }
    };

    // initiates communication bridge between audio input and wgpu
    let (bridge_sender, bridge_receiver) = mpsc::channel();
    bridge::init(
        bridge_receiver,
        bridge_sender.clone(),
        config.clone(),
    );
    audio::stream_input(
        audio_device,
        bridge_sender.clone(),
        config.clone(),
    );

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(String::from("audiovis"))
        .build(&event_loop).unwrap();

    // window configuration
    window.set_cursor_visible(!config.visual.hide_cursor);

    let mut state = pollster::block_on(State::new(
        &window,
        bridge_sender,
        config.clone(),
        bar_texture,
    ));

    if config.visual.fullscreen {
        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
    }
    window.set_always_on_top(config.visual.window_always_on_top);

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
                            // F for fullscreen
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::F),
                                ..
                            } => {
                                if window.fullscreen().is_none() {
                                    window.set_fullscreen(Some(Fullscreen::Borderless(None)))
                                } else {
                                    window.set_fullscreen(None)
                                }
                            }
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

