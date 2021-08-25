use std::sync::mpsc;
use std::thread;
use std::time::Instant;


pub enum Event {
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<f32>>),
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    gravity: f32,
) {
    let mut time: Instant = Instant::now();
    let mut buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match event {
                Event::Push(n) => {
                    let delta_time: u128 = time.elapsed().as_nanos();
                    time = Instant::now();
                    buffer = smooth_buffer(&mut buffer, n, gravity, delta_time);
                }
                Event::Consume(sender) => {
                    sender.send(buffer.clone()).unwrap();
                }
            },
            Err(e) => eprintln!(
                "an error occured while transmitting data between threads, {}",
                e
            ),
        };
    });
}


fn smooth_buffer(
    old_buffer: &mut Vec<f32>,
    new_buffer: Vec<f32>,
    gravity: f32,
    delta_time: u128,
) -> Vec<f32> {
    // buffering and time smoothing
    let mut output_buffer: Vec<f32> = Vec::new();
    let difference: i32 = new_buffer.len() as i32 - old_buffer.len() as i32;
    if difference > 0 {
        for _ in 0..difference {
            old_buffer.push(0.0);
        }
    }
    for i in 0..new_buffer.len() {
        if new_buffer[i] > old_buffer[i] {
            old_buffer[i] = new_buffer[i]
        }
        let mut gravity: f32 = ((gravity + 1.0) * 0.75) / (delta_time as f32 * 0.00000002);
        if gravity < 1.0 {
            gravity = 1.0;
        }
        output_buffer.push(old_buffer[i] / gravity);
    }
    return output_buffer;
}
