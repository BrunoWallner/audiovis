use std::sync::mpsc;
use std::thread;

pub enum Event {
    Push(Vec<f32>),
    PushFinal((Vec<Vec<f32>>, Vec<f32>)),
    Consume(mpsc::Sender<Vec<f32>>),
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    buffering: usize,
    smooth_size: u32,
    smooth_amount: u32,
    frequency: u32,
    low_frequency_threshold: u32,
    l_freq_scale_doub: u8,
    l_freq_volume_reduction: bool,
    l_freq_smoothing: u8,
    l_freq_smoothing_size: u32,
    l_freq_fading: f32,
) {
    let mut buffer: Vec<Vec<f32>> = Vec::new();
    let mut rounded_buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match event {
                Event::Push(n) => {
                    let sender = sender.clone();
                    let mut n = n.clone();
                    let mut buffer = buffer.clone();
                    thread::spawn(move || {
                        scale_low_frequencies(
                            &mut n,
                            l_freq_scale_doub,
                            frequency,
                            low_frequency_threshold,
                            l_freq_volume_reduction,
                            l_freq_smoothing,
                            l_freq_smoothing_size,
                            l_freq_fading,
                        );
                        n = smooth_buffer(&mut buffer, n, buffering, smooth_size, smooth_amount);
                        match sender.send(Event::PushFinal((buffer, n.clone()))) {
                            Ok(_) => (),
                            Err(e) => eprintln!("failed to send audio data to bridge, {}", e),
                        };
                    });
                }
                Event::Consume(sender) => {
                    sender.send(rounded_buffer.clone()).unwrap();
                }
                Event::PushFinal((buf, r_buf)) => {
                    buffer = buf;
                    rounded_buffer = r_buf;
                }
            },
            Err(e) => eprintln!(
                "an error occured while transmitting data between threads, {}",
                e
            ),
        };
    });
}

#[derive(Clone, Copy, Debug)]
struct Insertion {
    value: f32,
    position: usize,
}

fn scale_low_frequencies(
    buffer: &mut Vec<f32>,
    l_freq_scale: u8,
    frequency: u32,
    low_frequency_threshold: u32,
    volume_reduction: bool,
    smoothing: u8,
    smooth_size: u32,
    fading: f32,
) {
    let percentage: f32 = low_frequency_threshold as f32 / frequency as f32;
    let buffer_len = buffer.len();

    let mut scaled: usize = 0;
    for _ in 0..l_freq_scale {
        let low_freq_len: usize = (buffer_len as f32 * percentage) as usize + scaled;
        let mut position: usize = 0;
        for _ in 0..=(low_freq_len as f32 * fading) as u32 {
            let value: f32 = (buffer[position] + buffer[position + 1]) / 2.0;
            buffer.insert(position + 1, value);
            position += 2;
            scaled += 1;
        }
    }

    let low_freq_len: usize = (buffer_len as f32 * percentage) as usize + scaled;

    //smoothing
    for _ in 0..smoothing {
        for j in 0..low_freq_len {
            let mut y: f32 = 0.0;
            let mut smoothed: f32 = 0.0;
            for x in 0..smooth_size as usize {
                let place: usize = j + x;
                if place <= low_freq_len {
                    y += buffer[place as usize];
                    smoothed += 1.0;
                }
            }
            buffer[j] = y / smoothed;
        }
    }

    // volume
    if volume_reduction {
        for i in 0..low_freq_len {
            let percentage: f32 = (low_freq_len - i) as f32 / low_freq_len as f32;
            let calculated_volume: f32 = 1.0 - percentage * 0.85;
            buffer[i] *= calculated_volume;
        }
    }

    /* volume
    for i in 0..low_freq_len {
        let percentage: f32 = i as f32 / low_freq_len as f32;
        buffer[i] *= percentage / 1.025;
    }
    */
}

fn smooth_buffer(
    input_buffer: &mut Vec<Vec<f32>>,
    new_buffer: Vec<f32>,
    max_buffers: usize,
    smooth_size: u32,
    smooth_amount: u32,
) -> Vec<f32> {
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
                    smooth_buffer[i] += buffer[i];
                }
            }
        }

        for i in 0..smooth_buffer_length {
            output_buffer.push(smooth_buffer[i] / input_buffer.len() as f32);
        }

        input_buffer.pop();
        input_buffer.insert(0, output_buffer.clone());

        // horizontal smoothing
        for _ in 0..smooth_size {
            output_buffer.push(output_buffer[output_buffer.len() - 1]);
        }
        for _ in 0..smooth_amount {
            for j in 0..output_buffer.len() - smooth_size as usize {
                let mut y: f32 = 0.0;
                for x in 0..smooth_size {
                    y += output_buffer[j as usize + x as usize];
                }
                output_buffer[j] = y / smooth_size as f32;
            }
        }
        for _ in 0..smooth_size {
            output_buffer.pop();
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

        for _ in 0..smooth_size {
            output_buffer.pop();
        }

        return output_buffer;
    }
}
