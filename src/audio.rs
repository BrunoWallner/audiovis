use crate::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug)]
pub struct AudioDevice {
    pub name: String,
    pub host: String,
}

pub fn enumerate_devices() -> Result<Vec<AudioDevice>, String> {
    let mut buffer: Vec<AudioDevice> = Vec::new();
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        let host = match cpal::host_from_id(host_id) {
            Ok(h) => h,
            Err(_) => return Err(String::from("host is unavailable")),
        };
        let devices = match host.devices() {
            Ok(d) => d,
            Err(_) => return Err(String::from("devices are unavailable")),
        };
        for (_, device) in devices.enumerate() {
            let name: String = match device.name() {
                Ok(n) => n,
                Err(_) => String::from("INVALID_NAME"),
            };
            buffer.push(AudioDevice {
                name,
                host: String::from(host_id.name()),
            });
        }
    }
    Ok(buffer)
}

pub fn stream_input(
    input_device: String,
    bridge_sender: mpsc::Sender<bridge::Event>,
    m_freq: u32,
    pre_fft_windowing: bool,
    low_high_freq_ration: f32,
) {
    thread::spawn(move || {
        let host = cpal::default_host();

        let device = match if input_device == "default" {
            host.default_output_device()
        } else {
            host.devices().unwrap()
                .find(|x| x.name().map(|y| y == input_device).unwrap_or(false))
        } {
            Some(d) => d,
            None => {
                println!("could not find input device: {}", input_device);
                std::process::exit(1);
            }
        };

        println!("using device: {}", device.name().unwrap());

        // build either input or output config
        let config = match device.default_input_config() {
            Ok(c) => {
                println!("using input config");
                c
            },
            Err(_) => {
                match device.default_output_config() {
                    Ok(c) => {
                        println!("using output config");
                    c
                    },
                    Err(_) => {
                        println!("could not find any config for device");
                        std::process::exit(1);
                    }
                }
            }
        };
        println!("Default input config: {:?}", config);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| handle_input_data_f32(data, bridge_sender.clone(), m_freq, pre_fft_windowing, low_high_freq_ration),
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

fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<bridge::Event>, m_freq: u32, pre_fft_windowing: bool, low_high_freq_ration: f32) {
    let sender = sender.clone();
    let b = data.to_vec();
    thread::spawn(move || {
        let buffer: Vec<f32> = convert_buffer(b, m_freq, pre_fft_windowing, low_high_freq_ration); // pretty cpu heavy
        sender.send(bridge::Event::Push(buffer)).ok();
    });
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

pub fn convert_buffer(
    input_buffer: Vec<f32>,
    m_freq: u32, pre_fft_windowing: bool,
    low_high_freq_ration: f32,
) -> Vec<f32> {
    let mut input_buffer: Vec<f32> = input_buffer;
    if pre_fft_windowing {
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
        let amount: f32 = 0.1 / percentage.powf(low_high_freq_ration);
        output_buffer[i] /= amount;
    }

    // max frequency
    let percentage: f32 = m_freq as f32 / 20000.0;
    output_buffer[0..(output_buffer.len() as f32 * percentage) as usize].to_vec()
}
