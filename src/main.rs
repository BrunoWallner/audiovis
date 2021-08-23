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

mod mesh;

mod config;
mod audio;
mod wgpu_abstraction;
use wgpu_abstraction::State;

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

        .arg(Arg::with_name("device")
                    .short("d")
                    .long("device")
                    .takes_value(true)
                    .help("use another device"))

        .arg(Arg::with_name("generate_default_config")
                    .short("g")
                    .long("generate-default-config")
                    .takes_value(false)
                    .help("generates default configuration"))

        .arg(Arg::with_name("list_devices")
                    .short("l")
                    .long("list-devices")
                    .takes_value(false)
                    .help("enumerate and list through all available audio devices"))
        .get_matches();

    let config_path: &str = matches.value_of("config").unwrap_or("default");
    let input_device: String = String::from(matches.value_of("device").unwrap_or("default"));

    if matches.is_present("generate_default_config") {
        config::generate_default_config();
        println!("generated default config");
        std::process::exit(0);
    }
    if matches.is_present("list_devices") {
        let devices = match audio::enumerate_devices() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
        for device in devices.iter() {
            println!("[{}]\t{}", device.host, device.name);
        }
        std::process::exit(0);
    }

    let config = match config::get_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
            }
    };

    // initiates communication bridge between audio input and wgpu
    let (bridge_sender, bridge_receiver) = mpsc::channel();
    let sender_clone = bridge_sender.clone();
    bridge::init(
        bridge_receiver,
        sender_clone,
        config.processing.buffering,
        config.visual.smoothing_size,
        config.visual.smoothing_amount,
        config.visual.max_frequency,
        config.processing.low_frequency_threshold,
        config.processing.low_frequency_scale_doubling,
        config.processing.low_frequency_smoothing,
        config.processing.low_frequency_smoothing_size,
        config.processing.low_frequency_fading,
    );
    let config_clone = config.clone();
    let sender_clone = bridge_sender.clone();
    audio::stream_input(
        input_device,
        sender_clone,
        config_clone.visual.max_frequency,
        config_clone.audio.pre_fft_windowing,
        config_clone.processing.volume_compensation,
    );

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
        config.audio.volume_factoring,
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
                                if !window.fullscreen().is_some() {
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

