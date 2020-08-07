#![feature(test)]
extern crate test;

mod player;
mod gen;
use engine::{drawing, game};

const SCREEN_DIM: (usize, usize) = (1920,1080);//960, 528;
//const ASPECT_RATIO: f32 = 9.0/16.0;
//const SCREEN_WIDTH: usize = (SCREEN_HEIGHT as f32 / ASPECT_RATIO)as usize;
const TARGET_FPS: u64 = 60;

const GAME_TITLE: &str = "Untitled Game v0.002";
const ENABLE_DEBUG: bool = true;        //if debug can be toggled

const CHUNK_DIM: (usize, usize) = (256,256);
const GEN_RANGE: isize = 25;            //how far out to gen chunks
const SET_SEED: bool = true;            //if seed should be set

struct Mouse {
    coords: (isize, isize),
    sprite: drawing::Sprite,
}

impl Mouse {
    fn new() -> Self {
        Self {
            coords: (0,0),
            sprite: drawing::Sprite::load("sprites/mouse.png").unwrap().scale(4),
        }
    }
}


fn main() {
    let mut seed = 0;
    let world = gen::World::new_perlin(CHUNK_DIM, &mut seed, SET_SEED, GEN_RANGE);                                          //generate world
    let mut screen= drawing::Screen::new(SCREEN_DIM.0, SCREEN_DIM.1);                                                       //create blank screen buffer
    let mut player = player::Player::spawn((0,0), drawing::Spritesheet::load("sprites/america.gif", 500).unwrap());         //spawn player at 0,0
    let mut camera_coords: (isize, isize) = (0-(SCREEN_DIM.0 as isize/2),0+(SCREEN_DIM.1 as isize/2));                      //set camera location
    let mut mouse = Mouse::new();
    let mut debug_flag = false;

    let mut fpslock = game::FpsLock::create_lock(TARGET_FPS);                                                               //create fps lock obj

    let event_loop = game::EventLoop::new();                                                                                //create event loop obj
    let mut input = game::WinitInputHelper::new();                                                                          //create input helper obj
    let mut window = game::Window::init(GAME_TITLE, SCREEN_DIM.0, SCREEN_DIM.1, &event_loop);                               //create window, and pixels buffer
    window.fullscreen();
    window.window.set_cursor_visible(false);



    event_loop.run(move |event, _, control_flow| {                                                                          //start game loop
        fpslock.start_frame();                                                                                              //start frame for fps lock
        if let game::Event::RedrawRequested(_) = event {                                                                    //if redraw requested
            draw_screen(&mut screen, &world, &mut player, camera_coords, debug_flag, fpslock.get_fps(), seed, &mouse);      //draws new frame to screen buffer
            screen.flatten(window.pixels.get_frame());                                                                      //flatten screen to 1D for render
            window.pixels.render().unwrap();                                                                                //render

            fpslock.end_frame();
        }

        if input.update(event) {                                                                                            //handle input events on loop? not just on event

            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                                            //if esc pressed
                *control_flow = game::ControlFlow::Exit;                                                                    //exit
                return;
            }

            if input.key_held(game::VirtualKeyCode::W) {player.walk(player::Direction::Up)}
            if input.key_held(game::VirtualKeyCode::A) {player.walk(player::Direction::Left)}
            if input.key_held(game::VirtualKeyCode::S) {player.walk(player::Direction::Down)}
            if input.key_held(game::VirtualKeyCode::D) {player.walk(player::Direction::Right)}
            if input.key_pressed(game::VirtualKeyCode::Space){player.jump()}
            if input.key_pressed(game::VirtualKeyCode::LShift) {player.running = true}
            else if input.key_released(game::VirtualKeyCode::LShift){ player.running = false}
            if input.key_pressed(game::VirtualKeyCode::F3) {debug_flag = !debug_flag}
            if let Some(m) = input.mouse() {mouse.coords = (m.0 as isize, m.1 as isize)};

            if let Some(factor) = input.scale_factor_changed() {                                                            //if window dimensions changed
                window.hidpi_factor = factor;                                                                               //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                                    //if window resized
                window.pixels.resize(size.width, size.height);                                                              //resize pixel aspect ratio
            }

            do_updates(&mut camera_coords, &mut player, &world);
            window.window.request_redraw();                                                                                 //request frame redraw
        }
    });
}



///updates camera position based off player coords
fn update_camera(camera_coords: &mut (isize,isize), player: &mut player::Player) {
    let distance_x = player.coords.0 - camera_coords.0 - (SCREEN_DIM.0 as isize/2) + (player.sprite.get_dimensions().0/2); //calc x coord distance
    let distance_y = player.coords.1 - camera_coords.1 + (SCREEN_DIM.1 as isize/2) - (player.sprite.get_dimensions().1/2); //calc y coord distance
    let move_cam = |distance, camera: &mut isize| {                                 //closure that handles moving camera                                    KINDA JANK
        if distance < 25 && distance > -25 && distance != 0 {                       //if camera distance less than 25px from player and not on player
            if distance >= 0 {*camera+=1}                                           //move 1px positive if positive
            else {*camera-=1}                                                       //move 1px neg if neg
        }
        else {*camera+=distance/25}                                                 //if farther than 25px move distance/25
    };
    move_cam(distance_x, &mut camera_coords.0);                                     //move camera x
    move_cam(distance_y, &mut camera_coords.1);                                     //move camera y
}



///gets 2D vec of current frame to draw from 4D Vec
fn draw_screen(screen: &mut drawing::Screen, world: &gen::World, player: &mut player::Player, camera_coords: (isize, isize), debug_flag: bool, fps: usize, seed: u32, mouse: &Mouse) {
    world.get_screen(&mut screen.buf, camera_coords, SCREEN_DIM, CHUNK_DIM);                            //gets visible pixels from world as 2d vec
    screen.draw_sprite(&player.sprite.get_sprite(), screen.get_coords(player.coords, camera_coords));   //draw player sprite
    if ENABLE_DEBUG && debug_flag {                                                                     //if debug flag and debug enabled:
        screen.draw_debug_block(screen.get_coords((camera_coords.0 + (SCREEN_DIM.0 as isize/2), camera_coords.1 - (SCREEN_DIM.1 as isize/2)), camera_coords), 5, [255;4]);           //render debug block on camera
        //screen.draw_debug_box(screen.get_coords(player.coords, camera_coords),                          //render debug outline on player
        //                        (player.sprite.get_sprite().width,
        //                        player.sprite.get_sprite().height),
        //                        [255,0,0,0]);
        let hitbox = player.sprite.get_hitbox(player.coords).iter().map(|(x,y)| screen.get_coords((*x,*y), camera_coords)).collect::<Vec<(isize,isize)>>(); //calc hitbox on screen
        screen.draw_hitbox(hitbox, [0,255,0,0]);                                                        //draw hitbox
        draw_debug_screen(screen, player, camera_coords, fps, seed, CHUNK_DIM)                          //render debug screen
    }
    screen.draw_text((20,SCREEN_DIM.1-30), GAME_TITLE, 32.0, [255,255,255,0], drawing::DEBUG_FONT);     //render game title
    screen.draw_sprite(&mouse.sprite, mouse.coords);                                                    //draw mouse
}


///draws debug text
pub fn draw_debug_screen(screen: &mut drawing::Screen, player: &mut player::Player, camera_coords: (isize,isize), fps: usize, seed: u32, chunk_dim: (usize, usize)) {
    let size = 32.0;
    let color = [255,0,0,0];
    screen.draw_text((20,20), "DEBUG", size, color, drawing::DEBUG_FONT);
    let s = format!("{} FPS", fps);
    screen.draw_text((20,40), &s, size, color, drawing::DEBUG_FONT);
    let s = format!("Player: {}, {}", player.coords.0, player.coords.1);
    screen.draw_text((20,60), &s, size, color, drawing::DEBUG_FONT);
    let s = format!("Velocity: {:2.3}, {:2.3}", player.velocity.0, player.velocity.1);
    screen.draw_text((20,80), &s, size, color, drawing::DEBUG_FONT);
    let s = format!("Chunk: {}, {} in {}, {}", player.coords.0 % chunk_dim.0 as isize, player.coords.1 % chunk_dim.0 as isize,
                                            player.coords.0 / chunk_dim.0 as isize, player.coords.1 / chunk_dim.0 as isize,);
    screen.draw_text((20,100), &s, size, color, drawing::DEBUG_FONT);
    let s = format!("Camera: {}, {}", camera_coords.0, camera_coords.1);
    screen.draw_text((20,120), &s, size, color, drawing::DEBUG_FONT);
    let s = format!("Seed: {}", seed);
    screen.draw_text((20,140), &s, size, color, drawing::DEBUG_FONT);
}


fn do_updates(camera_coords: &mut (isize, isize), player: &mut player::Player, world: &gen::World) {
    player.update_location(&world, CHUNK_DIM);                                                          //update player location
    player.sprite.update();
    update_camera(camera_coords, player);                                                               //move camera towards player
}


//Chunk
//    coords: (i32,i32),
//    data: Vec<Vec<Particle>>
//         coords: (u8,u8)
//         data: Particle
//             rgba: [u8;4]

//loaded = index in world vec
//world = world coords
//chunk = chunk index
//inner = local index within chunk
//abs = absolute coords




//game currently uses 10%=11% cpu and 62mb memory OLD
//get_screen optimized from 135fps -> 145fps

//flatten from 145fps -> 235fps
//achieved 60fps 1920x1080!

//get_screen lagged with bigger screens
//1_000 1920x1080 optimized 13.760956ms -> 2.940932ms
//by 60fps -> 155fps by adding #[inline] above get_screen

//started optimizing at 135fps 960x529
//ended at 570fps 960x529 OR 155fps 1920x1080

//de-optimized when i passed get_screen non constant chunk dimensions. caused full div operations to be compiled and ruined inlining
//re-optimized by changing chunk and screen coords to tuples and passing directly instead of in struct



mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    #[bench]
    fn bench_draw_screen(b: &mut Bencher) {
        let mut seed = 0;
        let world = gen::World::new_perlin(CHUNK_DIM, &mut seed, SET_SEED, GEN_RANGE);
        let mut screen= drawing::Screen::new(SCREEN_DIM.0, SCREEN_DIM.1);
        let mut player = player::Player::spawn((0,0), drawing::Spritesheet::load("sprites/america.gif", 500).unwrap());
        let camera_coords: (isize, isize) = (0-(SCREEN_DIM.0 as isize/2),0+(SCREEN_DIM.1 as isize/2));
        let mouse = Mouse::new();
        let debug_flag = false;
        let fpslock = game::FpsLock::create_lock(TARGET_FPS);
        b.iter(||
            draw_screen(&mut screen, &world, &mut player, camera_coords, debug_flag, fpslock.get_fps(), seed, &mouse)
        );
    }

    #[bench]
    fn bench_get_screen(b: &mut Bencher) {
        let mut seed = 0;
        let world = gen::World::new_perlin(CHUNK_DIM, &mut seed, SET_SEED, GEN_RANGE);
        let mut screen= drawing::Screen::new(SCREEN_DIM.0, SCREEN_DIM.1);
        let camera_coords: (isize, isize) = (0-(SCREEN_DIM.0 as isize/2),0+(SCREEN_DIM.1 as isize/2));
        b.iter(||
            world.get_screen(&mut screen.buf, camera_coords, SCREEN_DIM, CHUNK_DIM)
        );
    }

    #[bench]
    fn bench_update_location(b: &mut Bencher) {
        let mut seed = 0;
        let world = gen::World::new_perlin(CHUNK_DIM, &mut seed, SET_SEED, GEN_RANGE);
        let mut player = player::Player::spawn((0,0), drawing::Spritesheet::load("sprites/america.gif", 500).unwrap());
        b.iter(||{
            player.walk(player::Direction::Right);
            player.update_location(&world, CHUNK_DIM);                                                        
        });
    }

    #[bench]
    fn bench_update_camera(b: &mut Bencher) {
        let mut player = player::Player::spawn((0,0), drawing::Spritesheet::load("sprites/america.gif", 500).unwrap());
        let mut camera_coords: (isize, isize) = (0-(SCREEN_DIM.0 as isize/2),0+(SCREEN_DIM.1 as isize/2));
        b.iter(||{
            player.walk(player::Direction::Right);
            update_camera(&mut camera_coords, &mut player);
        });
    }
}