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
            let mut previos_rise: bool = false;
            let mut previos_fall: bool = false;
            for i in 0..buffer.len() - 1 {
                let x1 = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let x2 = ((i + 1) as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let y1: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) * 0.05 ) - 1.0;
                let y2: f32 = volume_amplitude * ( (buffer[i + 1] as f32).powf(volume_factoring) * 0.05 ) - 1.0;

                let w = width * 1.125;

                let top_color1: [f32; 3] = [ top_color[0] * (y1 + 1.0),  top_color[1] * (y1 + 1.0),  top_color[2] * (y1 + 1.0)];
                let top_color2: [f32; 3] = [ top_color[0] * (y2 + 1.0),  top_color[1] * (y2 + 1.0),  top_color[2] * (y2 + 1.0)];

                // unclean code but works and should not impact performance
                if y1 < y2 {
                    if previos_fall {
                        vertices.push(Vertex { position: [x1,  y1, 0.0],   color: top_color1 });
                        vertices.push(Vertex { position: [x1 - w,  y1 - w, 0.0],   color: top_color1 });
                        vertices.push(Vertex { position: [x1 + w,  y1 - w, 0.0],   color: top_color1 });
                        vertices.push(Vertex { position: [x1,  y1 - w*2.0, 0.0],   color: top_color1 });

                        let i = vertices.len() as u16 - 4;
                        indices.push(i + 0);
                        indices.push(i + 1);
                        indices.push(i + 2);
                        indices.push(i + 3);
                        indices.push(i + 2);
                        indices.push(i + 1);
                    }
                    vertices.push(Vertex { position: [x1 + w,  y1 - w, 0.0],   color: top_color1 });
                    vertices.push(Vertex { position: [x1 - w,  y1 + w, 0.0],   color: top_color1 });
                    vertices.push(Vertex { position: [x2 - w,  y2 + w, 0.0],   color: top_color2 });
                    vertices.push(Vertex { position: [x2 + w,  y2 - w, 0.0],   color: top_color2 });

                    let i = vertices.len() as u16 - 4;
                    indices.push(i + 2);
                    indices.push(i + 1);
                    indices.push(i + 0);
                    indices.push(i + 2);
                    indices.push(i + 0);
                    indices.push(i + 3);

                    previos_rise = true;
                    previos_fall = false;
                }
                if y1 > y2 {
                    if previos_rise {
                        vertices.push(Vertex { position: [x1,  y1, 0.0],   color: top_color1 });
                        vertices.push(Vertex { position: [x1 + w,  y1 + w, 0.0],   color: top_color1 });
                        vertices.push(Vertex { position: [x1 - w,  y1 + w, 0.0],   color: top_color1 });
                        vertices.push(Vertex { position: [x1,  y1 + w*2.0, 0.0],   color: top_color1 });

                        let i = vertices.len() as u16 - 4;
                        indices.push(i + 0);
                        indices.push(i + 1);
                        indices.push(i + 2);
                        indices.push(i + 3);
                        indices.push(i + 2);
                        indices.push(i + 1);
                    }
                    vertices.push(Vertex { position: [x1 - w,  y1 - w, 0.0],   color: top_color1 });
                    vertices.push(Vertex { position: [x1 + w,  y1 + w, 0.0],   color: top_color1 });
                    vertices.push(Vertex { position: [x2 + w,  y2 + w, 0.0],   color: top_color2 });
                    vertices.push(Vertex { position: [x2 - w,  y2 - w, 0.0],   color: top_color2 });

                    let i = vertices.len() as u16 - 4;
                    indices.push(i + 2);
                    indices.push(i + 1);
                    indices.push(i + 0);
                    indices.push(i + 2);
                    indices.push(i + 0);
                    indices.push(i + 3);

                    previos_fall = true;
                    previos_rise = false;
                }
                if y1 == y2 {
                    vertices.push(Vertex { position: [x1,  y1 - w, 0.0],   color: top_color1 });
                    vertices.push(Vertex { position: [x1,  y1 + w, 0.0],   color: top_color1 });
                    vertices.push(Vertex { position: [x2,  y2 + w, 0.0],   color: top_color2 });
                    vertices.push(Vertex { position: [x2,  y2 - w, 0.0],   color: top_color2 });

                    let i = vertices.len() as u16 - 4;
                    indices.push(i + 2);
                    indices.push(i + 1);
                    indices.push(i + 0);
                    indices.push(i + 2);
                    indices.push(i + 0);
                    indices.push(i + 3);

                    previos_fall = false;
                    previos_rise = false;
                }
            }
        },
        _ => (),
    }
    return (vertices, indices);
}
