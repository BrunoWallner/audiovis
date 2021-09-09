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

        let (buffer_sender, buffer_receiver) = mpsc::channel();

        init_buffer_receiver(buffer_receiver, bridge_sender.clone(), config);

        let stream = match device_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &device_config.into(),
                move |data, _: &_| handle_input_data_f32(data, buffer_sender.clone()),
                err_fn,
            ).unwrap(),
            other => {
                panic!("Unsupported sample format {:?}", other);
            }
        };

        stream.play().unwrap();

        thread::park();
    });
}

// receives and buffers audiodata and converts it via fft if buffer size is big enough
fn init_buffer_receiver(receiver: mpsc::Receiver<Vec<f32>>, sender: mpsc::Sender<bridge::Event>, config: Config) {
    thread::spawn(move || {
        let mut buffer: Vec<f32> = Vec::new();

        loop  {
            let mut r = receiver.recv().unwrap();
            buffer.append(&mut r);
            if buffer.len() > config.processing.resolution as usize {
                let b = convert_buffer(buffer[0..config.processing.resolution as usize].to_vec(), config.clone());
                sender.send(bridge::Event::Push(b)).unwrap();
                buffer = Vec::new();
            }
        }
    });
}

fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<Vec<f32>>) {
    let sender = sender;
    sender.send(data.to_vec()).unwrap();
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

// most cpu intensive parts down here, could probably be improved

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
    for i in input_buffer.iter() {
        buffer.push(Complex {
            re: *i,
            im: 0.0,
        });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    for i in buffer.iter() {
        output_buffer.push(i.norm())
    }

    // remove mirroring
    let output_buffer = output_buffer[0..(output_buffer.len() as f32 * 0.25) as usize].to_vec();

    let mut output_buffer = normalize(output_buffer, config.processing.normalisation_factoring);

    scale_fav_frequencies(
        &mut output_buffer,
        config.processing.fav_frequency_range,
        config.processing.fav_frequency_doubling,
        config.processing.normalisation_factoring,
    );

    smooth(&mut output_buffer, config.visual.smoothing_amount, config.visual.smoothing_size);

    output_buffer
}

fn scale_fav_frequencies(buffer: &mut Vec<f32>, fav_freqs: [u32; 2], doubling: u16, factoring: f32) {
    let mut doubled: usize = 0;
    let buffer_len = buffer.len();
    for i in 0..doubling {
        let start_percentage: f32 = fav_freqs[0] as f32 / 20_000.0;
        let end_percentage: f32 = fav_freqs[1] as f32 / 20_000.0;

        let start_pos: usize = (buffer_len as f32 * start_percentage) as usize;
        let end_pos: usize = (buffer_len as f32 * end_percentage) as usize;

        let mut normalized_start_pos: usize = ((buffer_len as f32 / start_pos as f32).powf(factoring) * start_pos as f32) as usize;
        normalized_start_pos = (normalized_start_pos as f32 * (1.0 - ( ( (i + 1) as f32 / doubling as f32) * 0.25))) as usize; // for smoothing edge between non scaled and scaled freqs

        let normalized_end_pos: usize = ((buffer_len as f32 / end_pos as f32).powf(factoring) * end_pos as f32) as usize + doubled;

        let mut position: usize = normalized_start_pos;
        for _ in normalized_start_pos..normalized_end_pos {
            if position < buffer.len() - 1 {
                let value: f32 = (buffer[position] + buffer[position + 1]) / 2.0;

                buffer.insert(position + 1, value);
                position += 2;
                doubled += 1;
            }
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn normalize(buffer: Vec<f32>, factoring: f32) -> Vec<f32> {
    let buffer_len: usize = buffer.len();
    let mut output_buffer: Vec<f32> = vec![0.0; buffer_len];

    let mut start_pos: usize = 0;
    let mut end_pos: usize = 0;

    let mut pos_index: Vec<[usize; 2]> = Vec::new();

    for i in 0..buffer_len {
        let offset: f32 = (buffer_len as f32 / (i + 1) as f32).powf(factoring);
        if ((i as f32 * offset) as usize) < output_buffer.len() {
            // sets positions needed for future operations
            let pos: usize = (i as f32 * offset) as usize;
            start_pos = end_pos;
            end_pos = pos;
            pos_index.push([start_pos, end_pos]);

            // volume normalisation
            let mut y = buffer[i];
            y *= ((i + 1) as f32 / buffer_len as f32).powf(0.75);

            if output_buffer[pos] < y {
                output_buffer[pos] = y;
            }
        }
        if end_pos - start_pos > 1 && (end_pos - 1) < output_buffer.len() {
            // filling
            for s_p in (start_pos + 1)..end_pos {
                let percentage: f32 = (s_p - start_pos) as f32 / ((end_pos - 1) - start_pos) as f32;

                let mut y: f32 = 0.0;
                //(output_buffer[s_p] * (1.0 - percentage) ) + (output_buffer[end_pos] * percentage);
                y += output_buffer[start_pos] * (1.0 - percentage);
                y += output_buffer[end_pos] * percentage;
                output_buffer[s_p] = y;
            }
        }
    }

    output_buffer
}

fn smooth(
    buffer: &mut Vec<f32>,
    smoothing: u32,
    smoothing_size: u32,
) {
    for _ in 0..smoothing {
        for i in 0..buffer.len() - smoothing_size as usize {
            // reduce smoothing for higher freqs
            let percentage: f32 = i as f32 / buffer.len() as f32;
            let smoothing_size = (smoothing_size as f32 * (1.5 - percentage.powf(2.0))) as u32;

            let mut y = 0.0;
            for x in 0..smoothing_size as usize {
                y += buffer[i+x];
            }
            buffer[i] = y / smoothing_size as f32;
        }
    }
}


