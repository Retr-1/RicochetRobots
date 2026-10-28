use macroquad::{camera::Camera2D, prelude::*};

pub struct Layout {
    pub scale: f32,
    pub origin: Vec2,
    pub camera: Camera2D,
}
impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        let scale = (width / 440.).min(height / 860.);
        let origin = vec2((width - 440. * scale) / 2., (height - 860. * scale) / 2.);
        Self {
            scale,
            origin,
            camera: Camera2D {
                // Macroquad 0.4.13 already inverts screen Y in Camera::matrix.
                // Positive zoom keeps UI coordinates increasing down the screen.
                zoom: vec2(2. * scale / width, 2. * scale / height),
                target: vec2(220., 430.),
                ..Default::default()
            },
        }
    }
    pub fn pointer(&self, screen: Vec2) -> Vec2 {
        (screen - self.origin) / self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::camera::Camera;

    #[test]
    fn camera_keeps_text_upright_and_matches_touch_coordinates() {
        for (width, height) in [(440., 860.), (1080., 2400.), (2400., 1080.)] {
            let layout = Layout::new(width, height);
            let project = |p: Vec2| {
                let clip = layout.camera.matrix().transform_point3(p.extend(0.));
                vec2((clip.x + 1.) * width / 2., (1. - clip.y) * height / 2.)
            };
            let top_left = project(Vec2::ZERO);
            let top_right = project(vec2(440., 0.));
            let bottom_left = project(vec2(0., 860.));
            assert!(top_right.x > top_left.x, "X must not be mirrored");
            assert!(bottom_left.y > top_left.y, "Y must not be flipped");
            assert!((top_left - layout.origin).length() < 0.001);
            for point in [vec2(22., 42.), vec2(220., 430.), vec2(418., 840.)] {
                assert!((layout.pointer(project(point)) - point).length() < 0.001);
            }
        }
    }
}
