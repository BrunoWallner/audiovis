use crate::*;
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn init(sender: mpsc::Sender<bridge::Event>) {
    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();

    println!("Using input device: \"{}\"", input_device.name().unwrap());

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        sender.send(bridge::Event::Push(data.to_vec())).unwrap();
    };

    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn).unwrap();

    input_stream.play().unwrap();

    loop {}
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
