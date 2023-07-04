use wgpu::BufferViewMut;

pub struct Bullet {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

pub struct GameRegion {
    pub width: f32,
    pub height: f32,
    pub ways: u32,
    pub speed_per_frame: f32,
    pub angle: f32,
    pub a_angle: f32,
    pub a_a_angle: f32,
    pub bullets: Vec<Bullet>,
    pub half_bullet_width: f32,
    pub half_bullet_height: f32,
}

impl GameRegion {
    pub fn tick(&mut self) {
        let offset_delta = 360.0 / self.ways as f32;
        let mut cur_a = self.angle;

        let center_x = self.width / 2.0;
        let center_y = self.height / 2.0;


        for _ in 0..self.ways {
            let (dy, dx) = cur_a.to_radians().sin_cos();
            self.bullets.push(Bullet {
                x: center_x,
                y: center_y,
                dx,
                dy,
            });

            cur_a += offset_delta;
        }
        self.bullets.retain_mut(|bullet| {
            bullet.x += bullet.dx * self.speed_per_frame;
            bullet.y += bullet.dy * self.speed_per_frame;

            if bullet.x + self.half_bullet_width <= 0.0 || bullet.x - self.half_bullet_width >= self.width
                || bullet.y + self.half_bullet_height <= 0.0 || bullet.y - self.half_bullet_height >= self.height {
                return false;
            }

            true
        });

        self.a_angle += self.a_a_angle;
        self.a_angle %= 360.0;
        self.angle += self.a_angle;
        self.angle %= 360.0;
    }

    #[inline]
    pub fn upload(&self, mut buffer: BufferViewMut, bullet: &Bullet) {
        let result = bytemuck::cast_slice_mut::<_, [f32; 2]>(&mut buffer[..]);
        debug_assert_eq!(result.len(), 4);
        for (idx, point) in result.into_iter().enumerate() {
            let x = if (idx & 1) == 0 {
                bullet.x - self.half_bullet_width
            } else {
                bullet.x + self.half_bullet_width
            };

            let y = if idx < 2 {
                bullet.y - self.half_bullet_height
            } else {
                bullet.y + self.half_bullet_height
            };

            // [-1  x 1]       [0 width]
            let x = (x / self.width) * 2.0 - 1.0;

            //   ^  1         0
            //
            //   v -1         height
            let y = 1.0 - (y / self.height) * 2.0;
            point[0] = x;
            point[1] = y;
        }
    }
}
