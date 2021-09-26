use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Window,
    window::Fullscreen,
};
use wgpu::util::DeviceExt;
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
            smoothing_size: 50,
            smoothing_amount: 5,
            frequency_scale_range: [0, 3500],
            frequency_scale_amount: 3,
            buffering: 7,
            resolution: 3000,
            ..Default::default()
        }
    );
    let event_sender = audio_stream.get_event_sender();

    init_audio_sender(event_sender.clone());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(String::from("audiovis"))
        .build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(
        &window,
        event_sender,
    ));

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) {
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

