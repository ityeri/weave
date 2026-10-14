use glam::Vec2;
use rand::Rng;

pub struct Body {
    pub position: Vec2,
    pub prev_position: Vec2,
}

impl Body {
    pub fn random_range(radius: f32) -> Self {
        let mut rng = rand::rng();
        let position = Vec2::new(
            rng.random_range(-radius..radius),
            rng.random_range(-radius..radius),
        );
        Body {
            position: position,
            prev_position: position,
        }
    }
}
