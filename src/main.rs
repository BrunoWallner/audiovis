use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::sync::mpsc;

use audioviz;

mod graphics;
use graphics::wgpu_abstraction::State;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::thread;

fn main() {
    let audio_stream = audioviz::AudioStream::init(
        audioviz::Config {
            density_reduction: 10,
            smoothing_size: 5,
            smoothing_amount: 5,
            frequency_scale_range: [0, 3500],
            frequency_scale_amount: 3,
            max_frequency: 20_000,
            buffering: 5,
            resolution: 3000,
            ..Default::default()
        }
    );
    let event_sender = audio_stream.get_event_sender();

    init_audio_sender(event_sender.clone());
    init_auto_volume(event_sender.clone());

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

fn init_audio_sender(event_sender: mpsc::Sender<audioviz::Event>) {
    thread::spawn(move || {
        let host = cpal::default_host();

        let device = host.default_output_device().unwrap();

        let device_config =  device.default_output_config().unwrap();

        let stream = match device_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &device_config.into(),
                move |data, _: &_| handle_input_data_f32(data, event_sender.clone()),
                err_fn,
            ).unwrap(),
            other => {
                panic!("Unsupported sample format {:?}", other);
            }
        };

        stream.play().unwrap();

        // parks the thread so stream.play() does not get dropped and stops
        thread::park();
    });
}

fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<audioviz::Event>) {
    // sends the raw data to audio_stream via the event_sender
    sender.send(audioviz::Event::SendData(data.to_vec())).unwrap();
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

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

        let wanted_volume_amplitude = if average > 0.5 {
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

