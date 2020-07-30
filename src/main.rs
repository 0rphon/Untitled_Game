mod player;
mod gen;
use engine::{drawing, window};

use std::time;
use spin_sleep;

const SCREEN_HEIGHT: usize = 528;
const SCREEN_WIDTH: usize = 960;
//const ASPECT_RATIO: f32 = 9.0/16.0;
//const SCREEN_WIDTH: usize = (SCREEN_HEIGHT as f32 / ASPECT_RATIO)as usize;
const TARGET_FPS: u64 = 60;

const GAME_TITLE: &str = "Untitled Game v0.001";
const ENABLE_DEBUG: bool = true;        //if debug can be toggled

const CHUNK_WIDTH: usize = 256;
const CHUNK_HEIGHT: usize = 256;
const GEN_RANGE: isize = 4;             //how far out to gen chunks
const SET_SEED: bool = false;           //if seed should be set
const SEED: &str = "TESTSEED";          //seed to set



fn main() {
    let (mut rng, seed) = gen::get_rng(SET_SEED, SEED);                                                                     //get rng and display_seed
    let world = gen::init_world(&mut rng, GEN_RANGE, CHUNK_WIDTH, CHUNK_HEIGHT);                                            //generate world
    let mut screen: drawing::Screen = vec!(vec!([0;4]; SCREEN_WIDTH); SCREEN_HEIGHT);                                       //create blank screen buffer
    let mut player = player::Player::spawn((0,0), drawing::load_sprite("sprites/dude.png").unwrap());                       //spawn player at 0,0
    let mut camera_coords: (isize, isize) = (0,0);                                                                          //set camera location
    let mut debug_flag = false;

    let mut fps = 0;                                                                                                        //set var to record fps
    let mut frames = 0;                                                                                                     //set var to record frames this second
    let target_ft = time::Duration::from_micros(1000000 / TARGET_FPS);                                                      //set target fps
    let mut second_count = time::Instant::now();                                                                            //start second timer

    let (event_loop, mut input, window, mut hidpi_factor, mut pixels) = window::init(GAME_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);

    event_loop.run(move |event, _, control_flow| {                                                                          //start game loop
        let frame_time = time::Instant::now();                                                                              //set start of frame time
        if let window::Event::RedrawRequested(_) = event {                                                                          //if redraw requested
            draw_screen(&mut screen, &world, &mut player, camera_coords, debug_flag, fps, &seed);                           //draws new frame to screen buffer
            drawing::flatten(&screen, pixels.get_frame(), SCREEN_WIDTH);                                                    //render screen
            if pixels                                                                                                       //if rendering error
                .render()                                                                                                       
                .map_err(|e| panic!("pixels.render() failed: {}", e))
                .is_err() {
                *control_flow = window::ControlFlow::Exit;                                                                          //break
                return;
            }                

            frames+=1;                                                                                                      //inc frames this second
            if second_count.elapsed() > time::Duration::from_secs(1) {                                                      //if a second has elapsed
                fps = frames;                                                                                               //let fps = frames that occurred this second
                second_count = time::Instant::now();                                                                        //start new second
                frames = 0;                                                                                                 //reset frames this second to 0
            }
            
            if let Some(i) = (target_ft).checked_sub(frame_time.elapsed()) {                                                //if target frame time greater than this frames time
                spin_sleep::sleep(i);                                                                                       //sleep remainder
            }
        }
        
        if input.update(event) {                                                                                            //handle input events on loop? not just on event
            
            if input.key_pressed(window::VirtualKeyCode::Escape) || input.quit() {                                                  //if esc pressed
                *control_flow = window::ControlFlow::Exit;                                                                          //exit
                return;
            }

            if input.key_held(window::VirtualKeyCode::W) {player.walk(player::Direction::Up)}
            if input.key_held(window::VirtualKeyCode::A) {player.walk(player::Direction::Left)}
            if input.key_held(window::VirtualKeyCode::S) {player.walk(player::Direction::Down)}
            if input.key_held(window::VirtualKeyCode::D) {player.walk(player::Direction::Right)}
            if input.key_pressed(window::VirtualKeyCode::Space){player.jump()}
            if input.key_pressed(window::VirtualKeyCode::LShift) {player.running = true} 
            else if input.key_released(window::VirtualKeyCode::LShift){ player.running = false}
            if input.key_pressed(window::VirtualKeyCode::F3) {debug_flag = !debug_flag}
            
            if let Some(factor) = input.scale_factor_changed() {                                                            //if window dimensions changed
                hidpi_factor = factor;                                                                                      //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                                    //if window resized
                pixels.resize(size.width, size.height);                                                                     //resize pixel aspect ratio
            }

            //do world updates
            do_updates(&mut camera_coords, &mut player);
            window.request_redraw();                                                                                        //request frame redraw
        }
    });
}



///updates camera position based off player coords
fn update_camera(camera_coords: &mut (isize,isize), player: &mut player::Player) {
    let distance_x = player.coords.0 - camera_coords.0;         //calc x coord distance
    let distance_y = player.coords.1 - camera_coords.1;         //calc y coord distance
    let move_cam = |distance, camera: &mut isize| {             //closure that handles moving camera
        if distance < 25 && distance > -25 && distance != 0 {   //if camera distance less than 25px from player and not on player
            if distance >= 0 {*camera+=1}                       //move 1px positive if positive
            else {*camera-=1}                                   //move 1px neg if neg
        }
        else {*camera+=distance/25}                             //if farther than 25px move distance/25
    };
    move_cam(distance_x, &mut camera_coords.0);                 //move camera x
    move_cam(distance_y, &mut camera_coords.1);                 //move camera y
}



///gets 2D vec of current frame to draw from 4D Vec
fn draw_screen(screen: &mut drawing::Screen, world: &Vec<Vec<gen::Chunk>>, player: &mut player::Player, camera_coords: (isize, isize), debug_flag: bool, fps: usize, seed: &String) {
    gen::get_screen(screen, world,camera_coords, SCREEN_WIDTH, SCREEN_HEIGHT, CHUNK_WIDTH, CHUNK_HEIGHT);                                   //gets visible pixels from world as 2d vec
    drawing::draw_sprite(screen, &player.sprite, gen::get_screen_coords(player.coords, camera_coords, SCREEN_WIDTH, SCREEN_HEIGHT));        //draw player sprite
    if ENABLE_DEBUG && debug_flag {                                                                                                         //if debug flag and debug enabled:
        drawing::draw_debug_block(screen,                                                                                                   //render debug block on camera
                                    gen::get_screen_coords(camera_coords, 
                                                            camera_coords, 
                                                            SCREEN_WIDTH, 
                                                            SCREEN_HEIGHT), 
                                                            5, 
                                                            [255;4]);   
        drawing::draw_debug_outline(screen,                                                                                                 //render debug outline on player
                                    gen::get_screen_coords(player.coords, 
                                                            camera_coords, 
                                                            SCREEN_WIDTH, 
                                                            SCREEN_HEIGHT), 
                                    (player.sprite[0].len(), 
                                    player.sprite.len()), 
                                    [255,0,0,0]);     
        draw_debug_screen(screen, player, camera_coords, fps, seed, CHUNK_WIDTH)                                                            //render debug screen
    }                        
    drawing::draw_text(screen, (20,SCREEN_HEIGHT-30), GAME_TITLE, 16.0, [255,255,255,0], drawing::DEBUG_FONT);                              //render game title                         
}


///draws debug text
pub fn draw_debug_screen(screen: &mut drawing::Screen, player: &mut player::Player, camera_coords: (isize,isize), fps: usize, seed: &String, chunk_width: usize) {
    drawing::draw_text(screen, (20,20), "DEBUG", 16.0, [255,255,255,0], drawing::DEBUG_FONT);
    let s = format!("{} FPS", fps);
    drawing::draw_text(screen, (20,30), &s, 16.0, [255,255,255,0], drawing::DEBUG_FONT);
    let s = format!("Player: {}, {}", player.coords.0, player.coords.1);
    drawing::draw_text(screen, (20,40), &s, 16.0, [255,255,255,0], drawing::DEBUG_FONT);
    let s = format!("Velocity: {:2.3}, {:2.3}", player.velocity.0, player.velocity.1);
    drawing::draw_text(screen, (20,50), &s, 16.0, [255,255,255,0], drawing::DEBUG_FONT);
    let s = format!("Chunk: {}, {} in {}, {}", player.coords.0 % chunk_width as isize, player.coords.1 % chunk_width as isize, 
                                            player.coords.0 / chunk_width as isize, player.coords.1 / chunk_width as isize,);
    drawing::draw_text(screen, (20,60), &s, 16.0, [255,255,255,0], drawing::DEBUG_FONT);
    let s = format!("Camera: {}, {}", camera_coords.0, camera_coords.1);
    drawing::draw_text(screen, (20,70), &s, 16.0, [255,255,255,0], drawing::DEBUG_FONT);
    let s = format!("Seed: {}", seed);
    drawing::draw_text(screen, (20,80), &s, 16.0, [255,255,255,0], drawing::DEBUG_FONT);
}


fn do_updates(camera_coords: &mut (isize, isize), player: &mut player::Player) {
    player.update_location();                                                                           //update player location
    update_camera(camera_coords, player);                                                               //move camera towards player
}


//Chunk
//    coords: (i32,i32),
//    data: Vec<Vec<Particle>>
//         coords: (u8,u8)
//         data: Particle
//             rgba: [u8;4]

//gen_chunk = chunk index in vec
//world_chunk = world coordinates of chunk

//game currently uses 10%=11% cpu and 62mb memory

#[test]
fn test_it(){}