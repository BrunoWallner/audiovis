use crate::graphics::wgpu_abstraction::Vertex;

pub fn convert_to_buffer(
    buffer: Vec<Vec<f32>>,
    visualisation: String,
    width: f32,
    z_width: f32,
    volume_amplitude: f32,
    volume_factoring: f32,
) -> (Vec<Vertex>, Vec<u32>)  {

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    match visualisation.as_str() {
        "Bars" => {
            for z in 0..buffer.len() {
                let buffer_len = buffer[z].len();
                let width: f32 = 1.0 / buffer_len as f32 * width;
                for i in 0..buffer[z].len() {
                    let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                    let mut y: f32 = volume_amplitude * ( (buffer[z][i] as f32).powf(volume_factoring) * 0.05 ) - 1.0;
                    // max height
                    if y > 1.0 {
                        y = 0.0;
                    }
                    let mut texture_top_pos: f32 = y.powf(2.0);
                    // to prevent texture overflow
                    if texture_top_pos > 1.0 {
                        texture_top_pos = 1.0;
                    }
                    let z: f32 = z as f32 * -z_width;

                    vertices.push(Vertex { position: [x - width,  -1.0, z+z_width],   tex_coords:  [0.0, 1.0] });
                    vertices.push(Vertex { position: [x - width,  -1.0, z],   tex_coords:  [0.0, 1.0] });
                    vertices.push(Vertex { position: [x + width,  -1.0, z],   tex_coords:  [0.0, 1.0] });
                    vertices.push(Vertex { position: [x + width,  -1.0, z+z_width],   tex_coords:  [1.0, 1.0] });

                    vertices.push(Vertex { position: [x - width,  y, z+z_width],   tex_coords:  [0.0, texture_top_pos] });
                    vertices.push(Vertex { position: [x - width,  y, z],   tex_coords:  [0.0, texture_top_pos] });
                    vertices.push(Vertex { position: [x + width,  y, z],   tex_coords:  [0.0, texture_top_pos] });
                    vertices.push(Vertex { position: [x + width,  y, z+z_width],   tex_coords:  [1.0, texture_top_pos] });

                    let i = (vertices.len() - 8) as u32;
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
        },
        _ => (),
    }
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

