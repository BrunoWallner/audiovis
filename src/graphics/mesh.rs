use crate::graphics::wgpu_abstraction::Vertex;
use std::thread;
use std::sync::mpsc;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

pub fn from_buffer(
    buffer: Vec<Vec<f32>>,
    width: f32,
    z_width: f32,
    volume_amplitude: f32,
    volume_factoring: f32,
    multithreaded: u32,
) -> Mesh  {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut buffer_length_index: Vec<usize> = Vec::new();
    let mut length: usize = 0;
    for i in 0..buffer.len() {
        buffer_length_index.push(length);
        length += buffer[i].len();
    }
    drop(length);

    if multithreaded == 1 {
        for z in 0..buffer.len() {
            let buffer_len = buffer[z].len();
            let width: f32 = 1.0 / buffer_len as f32 * width;
            for i in 0..buffer_len {
                let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                let mut y: f32 = volume_amplitude * ( (buffer[z][i] as f32).powf(volume_factoring) * 0.01 );
                // max height important because texture overflow could appear
                if y > 1.0 {
                    y = 1.0;
                }
                let t_t_p: f32 = 1.0 - y; // texture_top_pos
                let t_l_p: f32 = z as f32 / buffer.len() as f32;
                let t_r_p: f32 = (z + 1) as f32 / buffer.len() as f32;
                let z: f32 = z as f32 * -z_width;

                y *= 2.0;
                y -= 1.0;


                vertices.append(&mut [
                    Vertex { position: [x - width,  -1.0, z+z_width],   tex_coords:  [t_l_p, 1.0] },
                    Vertex { position: [x - width,  -1.0, z],           tex_coords:  [t_r_p, 1.0] },
                    Vertex { position: [x + width,  -1.0, z],           tex_coords:  [t_r_p, 1.0] },
                    Vertex { position: [x + width,  -1.0, z+z_width],   tex_coords:  [t_l_p, 1.0] },

                    Vertex { position: [x - width,  y, z+z_width],      tex_coords:  [t_l_p, t_t_p] },
                    Vertex { position: [x - width,  y, z],              tex_coords:  [t_r_p, t_t_p] },
                    Vertex { position: [x + width,  y, z],              tex_coords:  [t_r_p, t_t_p] },
                    Vertex { position: [x + width,  y, z+z_width],      tex_coords:  [t_l_p, t_t_p] },
                ].to_vec());

                let i = (vertices.len() - 8) as u32;
                indices.append(&mut [
                    // front
                    i+0,
                    i+7,
                    i+4,
                    i+0,
                    i+3,
                    i+7,

                    // left
                    i+1,
                    i+4,
                    i+5,
                    i+1,
                    i+0,
                    i+4,

                    // right
                    i+3,
                    i+6,
                    i+7,
                    i+3,
                    i+2,
                    i+6,

                    // up
                    i+4,
                    i+6,
                    i+5,
                    i+4,
                    i+7,
                    i+6,
                ].to_vec());
            }
        }
    } else {
        // using multiple threads but its slower :(
        let (sender, receiver) = mpsc::channel();

        // distributes tasks to n threads

        let mut task_buffer: Vec<
            Vec<(Vec<f32>, usize)>
        > = Vec::new();

        for _ in 0..multithreaded {
            task_buffer.push(Vec::new());
        }

        let mut distribution_pos: usize = 0;
        let mut distributed_buffer_len: usize = 0;
        'distributing: loop {
            for j in 0..multithreaded as usize {
                let pos = distribution_pos + j;
                if buffer.len() > pos {
                    task_buffer[j].push((
                        buffer[pos].clone(),
                        pos,
                    ));
                    distributed_buffer_len += 1;
                } else {
                    break 'distributing;
                }
            }
            distribution_pos += multithreaded as usize;
        }

        // initiate every thread
        for j in 0..multithreaded as usize {
            let task = task_buffer[j].clone();
            let buffer_length_index = buffer_length_index.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                for t in 0..task.len() {
                    let buffer = &task[t].0;
                    let z = task[t].1;
                    let pre_vert_len = buffer_length_index[z] * 8;

                    let mut vertices: Vec<Vertex> = Vec::new();
                    let mut indices: Vec<u32> = Vec::new();

                    let buffer_len = buffer.len();
                    let width: f32 = 1.0 / buffer_len as f32 * width;
                    for i in 0..buffer_len {
                        let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                        let mut y: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) * 0.01 );
                        // max height important because texture overflow could appear
                        if y > 1.0 {
                            y = 1.0;
                        }
                        let texture_top_pos: f32 = 1.0 - (y * 0.5);
                        let z_orig = z;
                        let z: f32 = z as f32 * -z_width;

                        y *= 2.0;
                        y -= 1.0;

                        if z_orig == 0 {
                            vertices.append(&mut [
                                Vertex { position: [x - width,  -1.0, z+z_width],   tex_coords:  [0.0, 0.5] },
                                Vertex { position: [x - width,  -1.0, z],   tex_coords:  [0.0, 0.5] },
                                Vertex { position: [x + width,  -1.0, z],   tex_coords:  [0.0, 0.5] },
                                Vertex { position: [x + width,  -1.0, z+z_width],   tex_coords:  [1.0, 0.5] },

                                Vertex { position: [x - width,  y, z+z_width],   tex_coords:  [0.0, texture_top_pos - 0.5] },
                                Vertex { position: [x - width,  y, z],   tex_coords:  [0.0, texture_top_pos - 0.5] },
                                Vertex { position: [x + width,  y, z],   tex_coords:  [0.0, texture_top_pos - 0.5] },
                                Vertex { position: [x + width,  y, z+z_width],   tex_coords:  [1.0, texture_top_pos - 0.5] },
                            ].to_vec());
                        } else {
                            vertices.append(&mut [
                                Vertex { position: [x - width,  -1.0, z+z_width],   tex_coords:  [0.0, 1.0] },
                                Vertex { position: [x - width,  -1.0, z],   tex_coords:  [0.0, 1.0] },
                                Vertex { position: [x + width,  -1.0, z],   tex_coords:  [0.0, 1.0] },
                                Vertex { position: [x + width,  -1.0, z+z_width],   tex_coords:  [1.0, 1.0] },

                                Vertex { position: [x - width,  y, z+z_width],   tex_coords:  [0.0, texture_top_pos] },
                                Vertex { position: [x - width,  y, z],   tex_coords:  [0.0, texture_top_pos] },
                                Vertex { position: [x + width,  y, z],   tex_coords:  [0.0, texture_top_pos] },
                                Vertex { position: [x + width,  y, z+z_width],   tex_coords:  [1.0, texture_top_pos] },
                            ].to_vec());
                        }

                        let i = (vertices.len() - 8) as u32 + pre_vert_len as u32;
                        indices.append(&mut [
                            // front
                            i+0,
                            i+7,
                            i+4,
                            i+0,
                            i+3,
                            i+7,

                            // left
                            i+1,
                            i+4,
                            i+5,
                            i+1,
                            i+0,
                            i+4,

                            // right
                            i+3,
                            i+6,
                            i+7,
                            i+3,
                            i+2,
                            i+6,

                            // up
                            i+4,
                            i+6,
                            i+5,
                            i+4,
                            i+7,
                            i+6,
                        ].to_vec());
                    }
                    sender.send( ( (vertices, indices), z)).unwrap();
                }
            });
        }
        // receiving data from threads
        let mut receive_buffer: Vec<(Vec<Vertex>, Vec<u32>)> = Vec::new();
        for _ in 0..buffer.len() {
            receive_buffer.push( (Vec::new(), Vec::new()) );
        }

        let mut received: usize = 0;
        loop {
            // when both threads finished break
            if received == distributed_buffer_len {
                break;
            }
            let ( (v, i), z) = receiver.recv().unwrap();
            receive_buffer[z] = (v, i);
            received += 1;
        }

        // appending received data to vertices and indices
        // 60% of calculation time
        for i in 0..receive_buffer.len() {
            let mut v = receive_buffer[i].0.clone();
            let mut i = receive_buffer[i].1.clone();

            vertices.append(&mut v);
            indices.append(&mut i);
        }
    }
    return Mesh {vertices, indices};
}


/*
fn draw_line(
    point1: [f32; 2],
    point2: [f32; 2],
    width: f32,
    color: [f32; 3],
    vertex_len: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let x1: f32 = point1[0];
    let x2: f32 = point2[0];
    let y1: f32 = point1[1];
    let y2: f32 = point2[1];

    let dx = x2 - x1;
    let dy = y2 - y1;
    let l = dx.hypot (dy);
    let u = dx * width * 0.5 / l;
    let v = dy * width * 0.5 / l;

    vertices.push(Vertex { position: [x1 + v,  y1 - u, 0.0], color });
    vertices.push(Vertex { position: [x1 - v,  y1 + u, 0.0], color });
    vertices.push(Vertex { position: [x2 - v,  y2 + u, 0.0], color });
    vertices.push(Vertex { position: [x2 + v,  y2 - u, 0.0], color });

    indices.push(vertex_len + 2);
    indices.push(vertex_len + 1);
    indices.push(vertex_len + 0);
    indices.push(vertex_len + 2);
    indices.push(vertex_len + 0);
    indices.push(vertex_len + 3);

    return (vertices, indices);
}
*/

