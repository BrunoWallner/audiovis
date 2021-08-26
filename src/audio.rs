use crate::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::config::Config;

#[derive(Debug)]
pub struct AudioDevice {
    pub name: String,
    pub host: String,
}

#[derive(Debug)]
pub enum DeviceType {
    Input(),
    Output(),
}

pub fn stream_input(
    device_type: DeviceType,
    bridge_sender: mpsc::Sender<bridge::Event>,
    config: Config,
) {
    thread::spawn(move || {
        let host = cpal::default_host();

        let device =  match device_type {
            DeviceType::Input() => host.default_input_device().unwrap(),
            DeviceType::Output() => host.default_output_device().unwrap(),
        };
        println!("using device: {:#?}", device_type);

        // build either input or output config
        let device_config = match device_type {
            DeviceType::Input() => device.default_input_config().unwrap(),
            DeviceType::Output() => device.default_output_config().unwrap(),
        };

        let stream = match device_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &device_config.into(),
                move |data, _: &_| handle_input_data_f32(data, bridge_sender.clone(), config.clone()),
                err_fn,
            ).unwrap(),
            other => {
                panic!("Unsupported sample format {:?}", other);
            }
        };

        stream.play().unwrap();

        loop {}
    });
}

fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<bridge::Event>, config: Config) {
    let sender = sender.clone();
    let b = data.to_vec();
    thread::spawn(move || {
        let buffer: Vec<f32> = convert_buffer(b, config); // pretty cpu heavy
        sender.send(bridge::Event::Push(buffer)).ok();
    });
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

// most cpu intensive parts down here

fn apodize(buffer: Vec<f32>) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
}

pub fn convert_buffer(
    input_buffer: Vec<f32>,
    config: Config,
) -> Vec<f32> {
    let mut input_buffer: Vec<f32> = input_buffer;
    if config.audio.pre_fft_windowing {
        input_buffer = apodize(input_buffer)
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(input_buffer.len());

    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for i in 0..input_buffer.len() {
        buffer.push(Complex {
            re: input_buffer[i],
            im: 0.0,
        });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    let length: usize = buffer.len() / 2;
    for i in 0..length as usize {
        output_buffer.push(buffer[i].norm())
    }
    // *0.425 to cut off unwanted vector information that just mirrors itself and trims to exactly 20khz
    let mut output_buffer = output_buffer[0..(output_buffer.len() as f32 * 0.455) as usize].to_vec();

    // volume compensation
    let buffer_len = output_buffer.len();
    for i in 0..buffer_len {
        let percentage: f32 = i as f32 / buffer_len as f32;
        let amount: f32 = 0.1 / percentage.powf(config.processing.volume_compensation);
        output_buffer[i] /= amount;
    }
    // max frequency
    let percentage: f32 = config.visual.max_frequency as f32 / 20000.0;
    let mut output_buffer = output_buffer[0..(output_buffer.len() as f32 * percentage) as usize].to_vec();

    compensate_frequencies(&mut output_buffer, config.processing.frequency_compensation);
    output_buffer
}

fn compensate_frequencies(buffer: &mut Vec<f32>, compensation: f32) {
    let buffer_len = buffer.len();
    let mut smooth_step: f32 = 1.0;

    let mut scaled: usize = 0;

    'compensating: loop {
        let mut position: usize = 0;
        smooth_step *= compensation; // 3.5
        if smooth_step >= buffer.len() as f32 { break 'compensating }
        for _ in 0..=smooth_step as u32 {
            if position < buffer.len() - 1 {
                let value: f32 = (buffer[position] + buffer[position + 1]) / 2.0;
                buffer.insert(position + 1, value);
                position += 2;
                scaled += 1;
            }
        }
        // smoothing
        let interpolated_len: usize = buffer_len + scaled;
        for i in 0..interpolated_len {
            if i < buffer.len() - 1 {
                let value: f32 = (buffer[i] + (buffer[i+1])) / 2.0;
                buffer[i] = value;
            }
        }
    }
}
