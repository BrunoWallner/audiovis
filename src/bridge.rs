use std::sync::mpsc;
use std::thread;
use std::time::{Instant, Duration};
use ringbuf::RingBuffer;

pub enum Event {
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<f32>>),
}


pub fn init(receiver: mpsc::Receiver<Event>, buffering: usize, smooth_size: u32, smooth_amount: u32, bars: u32, should_scale: bool) {
    let mut buffer: Vec<Vec<f32>> = Vec::new();
    let mut rounded_buffer: Vec<f32> = Vec::new();
    let mut state: f32 = 0.0;
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Event::Push(mut n) => {
                        // 500 µs
                        if should_scale {n = scale(n, bars) }
                        n = smooth_buffer(&mut buffer, n, buffering, smooth_size, smooth_amount, bars);
                        add_end(&mut n);
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

#[derive(Debug)]
struct Interpolation {
    value: f32,
    place: usize,
}

fn add_end(buffer: &mut Vec<f32>) {
    /*
    for i in 0..(buffer.len() as f32 * 0.02) as usize {
        buffer.insert(0, buffer[i*2]);
    }
    */
    /*
    let mut start_value: f32 = 0.0;
    for i in 0..(buffer.len() as f32 * 0.05) as usize {
        start_value += buffer[i];
        buffer.insert(0, start_value / i as f32);
    }
    */
    for i in 0..(buffer.len() as f32 * 0.05) as usize {
        buffer.insert(0, buffer[0] / 2.0);
    }

    for i in 0..(buffer.len() as f32 * 0.05) as usize {
        buffer.push(buffer[buffer.len() - 1] / 2.0);
    }
}

/*
fn scale(buffer: &mut Vec<f32>, wanted_size: u32) {
    // upscaling
    let mut shift: bool = false;
    while wanted_size > buffer.len() as u32 {
        for i in 0..(buffer.len() as f32 / 10.0) as usize - 1 - 10 / 2 {
            let mut place: usize = i * 10;
            if shift { place += 10 / 2 }
            let value: f32 = (buffer[place] + buffer[place + 1]) / 2.0;
            buffer.insert(place + 1, value);
            shift = !shift;
        }
    }
    if buffer.len() > wanted_size as usize {
        // downscaling
        let difference: usize = buffer.len() - wanted_size as usize;
        let mut place: f32 = 0.0;
        let step = buffer.len() as f32 / difference as f32;
        for i in 0..difference - 1 {
            place += step;
            buffer.remove(place as usize);
            place -= 1.0;
        }
    }
}
*/

fn scale(buffer: Vec<f32>, wanted_size: u32) -> Vec<f32> {
    let mut output_buffer: Vec<f32> = Vec::new();
    let mut s: f32 = 0.0;
    for i in 0..wanted_size as usize {
        let mut step: f32 = (buffer.len() as f32 / wanted_size as f32);
        s += step;
        if buffer.len() > s as usize {
            output_buffer.push(buffer[s as usize]);
        } else {
            output_buffer.push(0.0);
        }
    }
    output_buffer
}


fn smooth_buffer(input_buffer: &mut Vec<Vec<f32>>, new_buffer: Vec<f32>, max_buffers: usize, smooth_size: u32, smooth_amount: u32, bars: u32) -> Vec<f32> {;
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
