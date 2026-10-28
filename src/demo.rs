use image::{Rgba, RgbaImage};

/// Synthetic sample used by the screenshot mode and manual demos.
pub fn landscape(width: u32, height: u32) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);
    let mut seed = 0xC0FFEE_u32;
    let sun_center = (width as f32 * 0.74, height as f32 * 0.30);
    let sun_radius = height as f32 * 0.17;

    for y in 0..height {
        let ny = y as f32 / height as f32;
        for x in 0..width {
            let nx = x as f32 / width as f32;
            let mut color = if ny < 0.56 {
                let t = ny / 0.56;
                if t < 0.65 {
                    lerp3([0.08, 0.13, 0.18], [0.27, 0.31, 0.39], smoothstep(t / 0.65))
                } else {
                    lerp3(
                        [0.27, 0.31, 0.39],
                        [0.95, 0.64, 0.44],
                        smoothstep((t - 0.65) / 0.35),
                    )
                }
            } else {
                lerp3(
                    [0.95, 0.64, 0.44],
                    [0.84, 0.49, 0.37],
                    smoothstep((ny - 0.56) / 0.24),
                )
            };

            let dx = x as f32 - sun_center.0;
            let dy = y as f32 - sun_center.1;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < sun_radius {
                let t = 1.0 - smoothstep((distance / sun_radius).powf(3.0));
                color = lerp3(color, [1.0, 0.95, 0.78], t);
            } else if distance < sun_radius * 2.8 {
                let t = (1.0 - smoothstep((distance - sun_radius) / (sun_radius * 1.8))).max(0.0);
                color = lerp3(color, [1.0, 0.85, 0.63], t * 0.18);
            }

            let ridge_far = 0.55 + 0.045 * (nx * 6.0 + 0.8).sin() + 0.025 * (nx * 15.0 + 2.0).sin();
            let ridge_near = 0.66 + 0.065 * (nx * 4.2 + 2.2).sin() + 0.02 * (nx * 13.0).sin();
            let ridge_front =
                0.82 + 0.035 * (nx * 3.0 + 4.3).sin() + 0.015 * (nx * 9.0 + 1.0).sin();
            if ny > ridge_far {
                color = lerp3(
                    [0.15, 0.23, 0.27],
                    [0.31, 0.39, 0.42],
                    0.22 + 0.3 * (1.0 - ny),
                );
            }
            if ny > ridge_near {
                color = lerp3(
                    [0.09, 0.16, 0.19],
                    [0.21, 0.29, 0.33],
                    0.16 + 0.3 * (1.0 - ny),
                );
            }
            if ny > ridge_front {
                color = lerp3(
                    [0.05, 0.10, 0.12],
                    [0.14, 0.20, 0.23],
                    0.10 + 0.22 * (1.0 - ny),
                );
            }

            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let grain = ((seed & 0xFF) as f32 / 255.0 - 0.5) * 0.025;
            image.put_pixel(
                x,
                y,
                Rgba([
                    channel(color[0] + grain),
                    channel(color[1] + grain),
                    channel(color[2] + grain),
                    255,
                ]),
            );
        }
    }
    image
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
