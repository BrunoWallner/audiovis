use crate::*;
use std::thread;
use std::convert::TryInto;
use rustfft::{FftPlanner, num_complex::Complex};

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
        let buffer: Vec<f32> = convert_buffer(data.to_vec());
        sender.send(bridge::Event::Push(buffer).try_into().unwrap()).unwrap();
    };

    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn).unwrap();

    input_stream.play().unwrap();

    loop {}
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn apodize(buffer: Vec<f32>) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
}

pub fn convert_buffer(input_buffer: Vec<f32>) -> Vec<f32> {
    let input_buffer = apodize(input_buffer);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(input_buffer.len());

    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for i in 0..input_buffer.len(){
        buffer.push(Complex {  re: input_buffer[i], im: 0.0 });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    let mut length: usize = buffer.len() / 2;
    for i in 0..length as usize  {
        output_buffer.push(buffer[i].norm())
    }
    output_buffer
}

fn un_mirror(buffer: &mut Vec<f32>) {
    for i in 0..buffer.len() / 2 {
        buffer[i] = (buffer[i] + buffer[buffer.len() - 1 - i]) / 2.0;
        buffer.pop();
    }
}
