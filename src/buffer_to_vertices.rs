use crate::wgpu_abstraction::Vertex;

pub fn create_mesh(
    buffer: Vec<f32>,
    visualisation: String,
    width: f32,
    volume_amplitude: f32,
    volume_factoring: f32,
    top_color: [f32; 3],
    bottom_color: [f32; 3],
) -> (Vec<Vertex>, Vec<u16>)  {

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let buffer_len = buffer.len();
    let width: f32 = 1.0 / buffer_len as f32 *   width;

    match visualisation.as_str() {
        "Bars" => {
            for i in 0..buffer.len() {
                let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                let y: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) * 0.05 ) - 1.0;

                let top_color: [f32; 3] = [ top_color[0] * (y + 1.0),  top_color[1] * (y + 1.0),  top_color[2] * (y + 1.0), ];

                vertices.push(Vertex { position: [x - width,  -1.0, 0.0],   color:  bottom_color });
                vertices.push(Vertex { position: [x - width,  y, 0.0],   color: top_color });
                vertices.push(Vertex { position: [x + width,  y, 0.0],   color: top_color });
                vertices.push(Vertex { position: [x + width,  -1.0, 0.0],   color:  bottom_color });

                let i = vertices.len() as u16 - 4;
                indices.push(i + 2);
                indices.push(i + 1);
                indices.push(i + 0);
                indices.push(i + 2);
                indices.push(i + 0);
                indices.push(i + 3);
            }
        },
        "Strings" => {
            let width = width * 2.5;
            for i in 0..buffer.len() - 1 {
                let x1: f32 = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let x2: f32 = ((i + 1) as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let y1: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) * 0.05 ) - 1.0;
                let y2: f32 = volume_amplitude * ( (buffer[i + 1] as f32).powf(volume_factoring) * 0.05 ) - 1.0;

                let color: [f32; 3] = [ top_color[0] * (y1 + 1.0),  top_color[1] * (y1 + 1.0),  top_color[2] * (y1 + 1.0)];

                let dx = x2 - x1;
                let dy = y2 - y1;
                let l = dx.hypot (dy);
                let u = dx * width * 0.5 / l;
                let v = dy * width * 0.5 / l;

                vertices.push(Vertex { position: [x1 + v,  y1 - u, 0.0], color });
                vertices.push(Vertex { position: [x1 - v,  y1 + u, 0.0], color });
                vertices.push(Vertex { position: [x2 - v,  y2 + u, 0.0], color });
                vertices.push(Vertex { position: [x2 + v,  y2 - u, 0.0], color });

                let i: u16 = vertices.len() as u16 - 4;
                indices.push(i + 2);
                indices.push(i + 1);
                indices.push(i + 0);
                indices.push(i + 2);
                indices.push(i + 0);
                indices.push(i + 3);
            }
        },
        _ => (),
    }
    return (vertices, indices);
}
