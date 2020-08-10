use engine::sprite;
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
    pub sprite: sprite::Spritesheet,
}

impl Player {
    pub fn spawn(coords: (isize, isize), sprite: sprite::Spritesheet) -> Player {
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

    //needs benchmark probably causing lag
    //there has to be a better way to do this
    pub fn update_location(&mut self, world: &World, chunk_dim: (usize, usize)) {
        let tx = self.coords.0 + self.velocity.0 as isize/10;                           //calculate target x
        let ty = (self.coords.1 + self.velocity.1 as isize/10)-15;                      //calculate target y

        let mut xrange: Vec<isize> = {                                                  //gets the range of movement on x axis
            if  tx > self.coords.0 {(self.coords.0..tx).collect()}                      //if moving positive returns movement range
            else {(tx..self.coords.0).rev().collect()}                                  //if moving negative then construct positive range and reverses it
        };
        let mut yrange: Vec<isize> = {
            if ty > self.coords.1 {(self.coords.1..ty).collect()}                       //gets range of movement on y axis
            else {(ty..self.coords.1).rev().collect()}                                  //if neg movement then constructs positive range and reverses
        };

        if yrange.len() > xrange.len() {                                                //if y movement bigger than x movement
            for _ in 0..yrange.len()-xrange.len() {xrange.push(tx)}                     //pad x to len of y
        }
        else if xrange.len() > yrange.len() {                                           //if x movement bigger than y movement
            for _ in 0..xrange.len()-yrange.len() {yrange.push(ty)}                     //pad y to len of x
        }

        for (wx,wy) in xrange.iter().zip(yrange) {                                      //iterate through coord pairs in range of movement
            if !world.check_collision(self.sprite.get_hitbox((*wx,wy)), chunk_dim) {    //if player hitbox of current sprite doesnt collide
                self.coords.0 = *wx;
                self.coords.1 = wy;
            }
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