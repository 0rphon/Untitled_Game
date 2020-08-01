use engine::drawing;
use crate::gen::*;

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct Player {
    pub health: usize,
    pub coords: (isize, isize),
    pub velocity: (f32, f32),
    pub max_velocity: f32,
    pub acceleration_speed: f32,
    pub deceleration_unit: f32,
    pub running: bool,
    pub sprite: drawing::Sprite,
}

impl Player {
    pub fn spawn(coords: (isize, isize), sprite: drawing::Sprite) -> Player {
        Player{
            health: 100,
            coords,
            velocity: (0.0,0.0),
            max_velocity: 50.0,
            acceleration_speed: 5.0,
            deceleration_unit: 20.0, //bad name. velocity -= velocity/decel_unit
            running: false,
            sprite,
        }
    }

    pub fn update_location(&mut self, world: &World) {                          //need to check for edge of hitbox
        let tmp_x = self.coords.0 + self.velocity.0 as isize/10;                //create fn called get_hitbox
        let tmp_y = (self.coords.1 + self.velocity.1 as isize/10)-15;           //then check every coord in vec
        if !world.check_collision((tmp_x,tmp_y)) {                              //also how do i fix catching? oh god i need to calc the curve dont i...
            self.coords.0 = tmp_x;
            self.coords.1 = tmp_y;
        }
        self.velocity.0 -= self.velocity.0/self.deceleration_unit;
        self.velocity.1 -= self.velocity.1/self.deceleration_unit;
    }

    pub fn walk(&mut self, direction: Direction) {
        match direction {
            Direction::Right => {
                if self.velocity.0 < self.max_velocity || self.running {
                    self.velocity.0 += self.acceleration_speed
                }
            },
            Direction::Left => {
                if self.velocity.0*-1.0 < self.max_velocity || self.running {
                    self.velocity.0 -= self.acceleration_speed
                }
            },
            Direction::Up => {
                if self.velocity.1 < self.max_velocity || self.running {
                    self.velocity.1 += self.acceleration_speed
                }
            },
            Direction::Down => {
                if self.velocity.1*-1.0 < self.max_velocity || self.running {
                    self.velocity.1 -= self.acceleration_speed
                }
            },
        }
    }

    pub fn jump(&mut self) {
        self.velocity.1+=300.0;
    }
}