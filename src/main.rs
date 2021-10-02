use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use audioviz;

mod graphics;
use graphics::wgpu_abstraction::State;
mod config;
pub use config::Visualisation;
mod audio;
use audio::*;

use clap::{Arg, App};

fn main() {
    let matches = App::new("audiovis")
    .version("0.1.0")
    .author("Luca Biendl <b.lucab1211@gmail.com>")
    .about("tool to visualize audio")
    .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("use custom configuration"))

    .arg(Arg::with_name("iter_devices")
                .short("i")
                .long("iter-devices")
                .takes_value(false)
                .help("iterate trough all available devices"))
                
    .arg(Arg::with_name("input_device")
                .long("input-device")
                .takes_value(true)
                .help("use specific input device"))

    .arg(Arg::with_name("output_device")
                .long("output-device")
                .takes_value(true)
                .help("use specific output device"))

    .arg(Arg::with_name("generate_default_config")
                .short("g")
                .long("generate-default-config")
                .takes_value(false)
                .help("generates default configuration"))

    .get_matches();

    let audio_device: AudioDevice = 
        //if matches.value_of("input_device").unwrap_or("0").parse().unwrap();
        if matches.is_present("input_device") {
            AudioDevice::Input(matches.value_of("input_device").unwrap_or("0").parse().unwrap())
        }
        else if matches.is_present("output_device") {
            AudioDevice::Output(matches.value_of("output_device").unwrap_or("0").parse().unwrap())
        } else {
            AudioDevice::Output(0)
        };

    if matches.is_present("iter_devices") {
        iter_audio_devices();
        std::process::exit(0);
    }

    let audio_stream = audioviz::AudioStream::init(
        audioviz::Config {
            density_reduction: 0,
            smoothing_size: 20,
            smoothing_amount: 5,
            frequency_scale_range: [0, 1000],
            frequency_scale_amount: 2,
            max_frequency: 20_000,
            buffering: 7,
            resolution: 2048,
            volume: 0.5,
            ..Default::default()
        }
    );
    let event_sender = audio_stream.get_event_sender();

    init_audio_sender(event_sender.clone(), audio_device);
    //init_auto_volume(event_sender.clone());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(&window, event_sender));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) { // UPDATED!
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            winit::event::Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
/*
fn init_auto_volume(event_sender: mpsc::Sender<audioviz::Event>) {
    thread::spawn(move || loop {
        let (tx, rx) = mpsc::channel();
        event_sender.send(audioviz::Event::RequestConfig(tx)).unwrap();
        let config = rx.recv().unwrap();

        let (tx, rx) = mpsc::channel();
        event_sender.send(audioviz::Event::RequestData(tx)).unwrap();
        let data = rx.recv().unwrap();

        let mut average: f32 = 0.0;
        for i in data.iter() {
            if *i > average {
                average = *i;
            }
        }

        let wanted_volume_amplitude = if average > 0.25 {
            config.volume - 0.01
        } else {
            config.volume + 0.01
        };

        let wanted_config: audioviz::Config = audioviz::Config {
            volume: wanted_volume_amplitude,
            ..config
        };

        event_sender.send(audioviz::Event::SendConfig(wanted_config)).unwrap();

        thread::sleep(std::time::Duration::from_millis(50));
    });
}
*/

