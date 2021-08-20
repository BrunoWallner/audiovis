use crate::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::thread;
use std::time::Duration;

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
            host.default_input_device()
        } else {
            host.input_devices().unwrap()
                .find(|x| x.name().map(|y| y == input_device).unwrap_or(false))
        } {
            Some(d) => d,
            None => {
                println!("could not find input device: {}", input_device);
                std::process::exit(1);
            }
        };

        println!("using input devices: {}", device.name().unwrap());

        let config: cpal::StreamConfig = device.default_input_config().unwrap().into();

        let (tx, rc) = mpsc::channel();

        let input_data_fn =
            move |data: &[f32], _: &cpal::InputCallbackInfo| match tx.send(data.to_vec()) {
                Ok(_) => (),
                Err(_) => thread::sleep(Duration::from_millis(100)),
            };

        let input_stream = device
            .build_input_stream(&config, input_data_fn, err_fn)
            .unwrap();


        instruction_receiver(rc, bridge_sender, m_freq, pre_fft_windowing, low_high_freq_ration);

        input_stream.play().unwrap();

        loop {}
    });
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn instruction_receiver(
    receiver: mpsc::Receiver<Vec<f32>>,
    sender: mpsc::Sender<bridge::Event>,
    m_freq: u32,
    pre_fft_windowing: bool,
    low_high_freq_ration: f32,
) {
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(b) => {
                let sender_clone = sender.clone();
                thread::spawn(move || {
                    let buffer: Vec<f32> = convert_buffer(b.to_vec(), m_freq, pre_fft_windowing, low_high_freq_ration);
                    sender_clone.send(bridge::Event::Push(buffer)).unwrap();
                });
            }
            Err(_) => thread::sleep(Duration::from_millis(500)),
        }
    });
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
