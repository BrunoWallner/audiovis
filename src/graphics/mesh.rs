use crate::graphics::wgpu_abstraction::Vertex;
use std::thread;
use std::sync::mpsc;

pub fn convert_to_buffer(
    mut buffer: Vec<Vec<f32>>,
    width: f32,
    z_width: f32,
    volume_amplitude: f32,
    volume_factoring: f32,
    multithreaded: bool,
    resolution_drop: f32,
    max_res_drop: u16,
    res_drop_z_factoring: f32,
) -> (Vec<Vertex>, Vec<u32>)  {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    if !multithreaded {
        for z in 0..buffer.len() {
            if resolution_drop > 0.0 {
                let mut amount = ( (z as f32).powf(res_drop_z_factoring) * resolution_drop * 0.1) as usize;
                if amount > max_res_drop as usize {
                    amount = max_res_drop as usize;
                }
                for _ in 0..amount {
                    let mut pos: usize = 1; // potential shifting but idk
                    loop {
                        if buffer[z].len() > pos + 1 {
                            if buffer[z][pos] < buffer[z][pos+1] {
                                buffer[z].remove(pos);
                            } else {
                                buffer[z].remove(pos+1);
                            }
                            pos += 2;
                        } else {
                            break;
                        }
                    }
                }
            }

            let buffer_len = buffer[z].len();
            let width: f32 = 1.0 / buffer_len as f32 * width;
            for i in 0..buffer_len {
                let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                let mut y: f32 = volume_amplitude * ( (buffer[z][i] as f32).powf(volume_factoring) * 0.01 );
                // max height important because texture overflow could appear
                if y > 1.0 {
                    y = 1.0;
                }
                let texture_top_pos: f32 = 1.0 - (y * 0.25);
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
        // using 2 threads but its slower :(
        let buffer_len = buffer.len();
        let (sender, receiver) = mpsc::channel();
        let mut receive_buffer: Vec<(Vec<Vertex>, Vec<u32>)> = Vec::new();

        // because 2 threads
        for _ in 0..2 {
            receive_buffer.push(
                (
                    Vec::new(),
                    Vec::new(),
                )
            );
        }
        for j in 0..2 {
            // number of vertices for second thread
            let start_size: usize =
            {
                let mut size: usize = 0;
                for i in (buffer.len() / 2)..buffer.len() {
                    size += buffer[i].len() * 8;
                }
                size
            };
            let buffer = buffer.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                let mut vertices: Vec<Vertex> = Vec::new();
                let mut indices: Vec<u32> = Vec::new();

                let start_pos = match j {
                    0 => 0,
                    1 => buffer_len / 2,
                    _ => panic!("invalid thread count on mesh creation"),
                };
                let end_pos = match j {
                    0 => buffer_len / 2,
                    1 => buffer_len,
                    _ => panic!("invalid thread count on mesh creation"),
                };
                let num_vertices = match j {
                    0 => 0,
                    1 => start_size,
                    _ => panic!("invalid thread count on mesh creation"),
                };

                for z in start_pos..end_pos {
                    let buffer_len = buffer[z].len();
                    let width: f32 = 1.0 / buffer_len as f32 * width;
                    for i in 0..buffer[z].len() {
                        let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                        let mut y: f32 = volume_amplitude * ( (buffer[z][i] as f32).powf(volume_factoring) * 0.01 ) - 1.0;
                        // max height
                        if y > 0.0 {
                            y = 0.0;
                        }
                        let mut texture_top_pos: f32 = 1.0 - (y + 1.0);
                        let z: f32 = z as f32 * -z_width;

                        // to prevent texture overflow
                        if texture_top_pos > 1.0 {
                            texture_top_pos = 1.0;
                        }
                        if texture_top_pos < 0.0 {
                            texture_top_pos = 0.0;
                        }

                        vertices.push(Vertex { position: [x - width,  -1.0, z+z_width],   tex_coords:  [0.0, 1.0] });
                        vertices.push(Vertex { position: [x - width,  -1.0, z],   tex_coords:  [0.0, 1.0] });
                        vertices.push(Vertex { position: [x + width,  -1.0, z],   tex_coords:  [0.0, 1.0] });
                        vertices.push(Vertex { position: [x + width,  -1.0, z+z_width],   tex_coords:  [1.0, 1.0] });

                        vertices.push(Vertex { position: [x - width,  y, z+z_width],   tex_coords:  [0.0, texture_top_pos] });
                        vertices.push(Vertex { position: [x - width,  y, z],   tex_coords:  [0.0, texture_top_pos] });
                        vertices.push(Vertex { position: [x + width,  y, z],   tex_coords:  [0.0, texture_top_pos] });
                        vertices.push(Vertex { position: [x + width,  y, z+z_width],   tex_coords:  [1.0, texture_top_pos] });

                        let i = (vertices.len() - 8 + num_vertices) as u32;
                        indices.push(i+0);
                        indices.push(i+7);
                        indices.push(i+4);
                        indices.push(i+0);
                        indices.push(i+3);
                        indices.push(i+7);

                        indices.push(i+1);
                        indices.push(i+4);
                        indices.push(i+5);
                        indices.push(i+1);
                        indices.push(i+0);
                        indices.push(i+4);

                        indices.push(i+2);
                        indices.push(i+5);
                        indices.push(i+6);
                        indices.push(i+2);
                        indices.push(i+1);
                        indices.push(i+5);

                        indices.push(i+3);
                        indices.push(i+6);
                        indices.push(i+7);
                        indices.push(i+3);
                        indices.push(i+2);
                        indices.push(i+6);

                        indices.push(i+0);
                        indices.push(i+1);
                        indices.push(i+2);
                        indices.push(i+0);
                        indices.push(i+2);
                        indices.push(i+3);

                        indices.push(i+4);
                        indices.push(i+6);
                        indices.push(i+5);
                        indices.push(i+4);
                        indices.push(i+7);
                        indices.push(i+6);
                    }
                }
                sender.send( ( (vertices, indices), j)).unwrap();
            });
        }
        // receiving data from threads
        let mut received: usize = 0;
        loop {
            // when both threads finished break
            if received == 2 {
                break;
            }
            let ( (v, i), j) = receiver.recv().unwrap();
            receive_buffer[j] = (v, i);
            received += 1;
        }

        // appending received data to vertices and indices
        for i in 0..receive_buffer.len() {
            let mut v = receive_buffer[i].0.clone();
            let mut i = receive_buffer[i].1.clone();

            vertices.append(&mut v);
            indices.append(&mut i);
        }
    }
    //println!("created mesh for {} bars in {} µs", total_bars, now.elapsed().as_micros());
    return (vertices, indices);
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

