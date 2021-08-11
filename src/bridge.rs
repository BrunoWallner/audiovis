use std::sync::mpsc;
use std::thread;
use std::time::{Instant, Duration};
use ringbuf::RingBuffer;

pub enum Event {
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<f32>>),
}


pub fn init(receiver: mpsc::Receiver<Event>, buffering: usize, smooth_size: u32, smooth_amount: u32, frequency: u32, low_frequency_threshold: u32, l_freq_scale_doub: u8) {
    let mut buffer: Vec<Vec<f32>> = Vec::new();
    let mut rounded_buffer: Vec<f32> = Vec::new();
    let mut state: f32 = 0.0;
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Event::Push(mut n) => {
                        // 500 µs
                        scale_low_frequencies(&mut n, l_freq_scale_doub, frequency, low_frequency_threshold);
                        n = smooth_buffer(&mut buffer, n, buffering, smooth_size, smooth_amount);
                        rounded_buffer = n;
                    }
                    Event::Consume(sender) => {
                        sender.send(rounded_buffer.clone()).unwrap();
                    }
                }
            },
            Err(_) => (),
        };
    });
}

#[derive(Clone, Copy, Debug)]
struct Insertion {
    value: f32,
    position: usize,
}

fn scale_low_frequencies(buffer: &mut Vec<f32>, l_freq_scale: u8, frequency: u32, low_frequency_threshold: u32) {
    let percentage: f32 = low_frequency_threshold as f32 /  frequency as f32;
    for _ in 0..l_freq_scale {
        let mut position: usize = 0;
        for _ in 0..(buffer.len() as f32 * percentage) as usize {
            position += 1;
            let value: f32 = (buffer[position] + buffer[position + 1]) / 2.0;
            buffer.insert(position + 1, value);
            position += 1;
        }


        // extra smoothing and transition
        for i in 0..(buffer.len() as f32 * percentage * 1.1) as usize {
            buffer[i] = (buffer[i] + buffer[i+1]) / 2.0;
        }

    }
    for i in 0..(buffer.len() as f32 * 0.015) as usize {
        buffer.insert(0, buffer[0] / 1.25);
    }
}

fn smooth_buffer(input_buffer: &mut Vec<Vec<f32>>, new_buffer: Vec<f32>, max_buffers: usize, smooth_size: u32, smooth_amount: u32) -> Vec<f32> {;
    // buffering and time smoothing
    if max_buffers > 0 {
        let mut output_buffer: Vec<f32> = Vec::new();
        input_buffer.insert(0, new_buffer);

        //input_buffer.push(new_buffer);
        if input_buffer.len() > max_buffers {
            input_buffer.pop();
        }

        let mut smooth_buffer: Vec<f32> = Vec::new();
        let mut smooth_buffer_length: usize = 0;
        for buffer in input_buffer.iter() {

            if smooth_buffer_length < buffer.len() {
                smooth_buffer_length = buffer.len();
                for _ in 0..buffer.len() - smooth_buffer.len() {
                    smooth_buffer.push(0.0);
                }
            }
            for i in 0..buffer.len() {
                if smooth_buffer_length > i {
                    //output_buffer[i] = (output_buffer[i] + buffer[i]) / 2.0;
                    smooth_buffer[i] += (buffer[i]);
                }
            }

        }

        for i in 0..smooth_buffer_length {
            output_buffer.push(smooth_buffer[i] / input_buffer.len() as f32);
        }

        input_buffer.pop();
        input_buffer.insert(0, output_buffer.clone());

        // horizontal smoothing
        for _ in 0..smooth_amount {
            for j in 0..output_buffer.len() - smooth_size as usize {
                let mut y: f32 = 0.0;
                for x in 0..smooth_size {
                    y += output_buffer[j as usize + x as usize];
                }
                output_buffer[j] = y / smooth_size as f32;
            }
            // smooth the last bit
            let output_buffer_len = output_buffer.len() as usize - 1;
            for j in 0..smooth_size as usize {
                let mut y: f32 = 0.0;
                for i in 0..smooth_size as usize {
                    y += output_buffer[output_buffer_len - i];
                }
                output_buffer[output_buffer_len - smooth_size as usize + j] = y / smooth_size as f32;
            }
        }

        return output_buffer;
    } else {
        // horizontal smoothing
        let mut output_buffer = new_buffer;
        for _ in 0..smooth_amount {
            for j in 0..output_buffer.len() - smooth_size as usize {
                let mut y: f32 = 0.0;
                for x in 0..smooth_size {
                    y += output_buffer[j as usize + x as usize];
                }
                output_buffer[j] = y / smooth_size as f32;
            }
        }

        return output_buffer;
    }
}
