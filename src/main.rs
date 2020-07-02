mod drawing;
mod gen;

use std::{thread, time};


use log::error;
use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const SCREEN_HEIGHT: usize = 528;
const SCREEN_WIDTH: usize = 960;
//const ASPECT_RATIO: f32 = 9.0/16.0;
//const SCREEN_WIDTH: usize = (SCREEN_HEIGHT as f32 / ASPECT_RATIO)as usize;
const TARGET_FPS: u64 = 70;    //VSYNC NOT ACCURATE

const GAME_TITLE: &str = "Untitled Game v0.001";
const ENABLE_DEBUG: bool = true;                                    //if debug can be toggled
const DEBUG_FONT: &[u8; 19836] = include_bytes!("fonts/debug.ttf");

const CHUNK_WIDTH: usize = 256;
const CHUNK_HEIGHT: usize = 256;
const GEN_RANGE: isize = 4;             //how far out to gen chunks
const SET_SEED: bool = true;            //if seed should be set
const SEED: &str = "TESTSEED";          //seed to set



fn main() {
    let mut frames = 0;                                                                                         //frame counter
    let mut fps_time = clock_ticks::precise_time_s();                                                           //keeps track of when a second passes
    let mut fps = 0;                                                                                            //stores last seconds fps
    let frame_time = 1000000 / TARGET_FPS;                                                                      //target fps

    let event_loop = EventLoop::new();                                                                          //create event loop obj
    let mut input = WinitInputHelper::new();                                                                    //create WinitIH obj
    let window = {                                                                                              //create window obj
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Prototype Engine")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();                                                               //get window dimensions

    let mut pixels = {                                                                                          //create pixel buffer
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
    };

    let (mut rng, seed) = gen::get_rng(SET_SEED, SEED);                                                         //get rng and display_seed
    let world = gen::init_world(&mut rng);                                                                      //generate world
    let mut player_coords: (isize, isize) = (128,-128);                                                         //set player location
    let mut camera_coords: (isize, isize) = (128,-128);                                                         //set camera location
    let mut debug_flag = false;

    event_loop.run(move |event, _, control_flow| {                                                              //start game loop
        let frame_start = clock_ticks::precise_time_s();                                                        //get current loop start time                           VSYNC NOT ACCURATE
        if let Event::RedrawRequested(_) = event {                                                              //if redraw requested
            drawing::draw(draw_screen(&world, player_coords, camera_coords, debug_flag, fps, &seed), pixels.get_frame());     //get screen then render screen
            if pixels                                                                                           //if rendering error
                .render()                                                                                                       
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err() {
                *control_flow = ControlFlow::Exit;                                                              //break
                return;
            }                
            frames+=1;                                                                                          //inc this seconds frame counter

            if (clock_ticks::precise_time_s() - fps_time) >= 1.0 {                                              //if second has passed since last second
                fps = frames;                                                                                   //fps = this seconds frames
                fps_time = clock_ticks::precise_time_s();                                                       //reset second time
                frames = 0;                                                                                     //reset second frames
            }
            
            match (frame_time).checked_sub(((clock_ticks::precise_time_s() - frame_start) * 1000000.0) as u64) {//if frame took less than target fps time               VSYNC NOT ACCURATE
                Some(i) => {thread::sleep(time::Duration::from_micros(i))}                                      //sleep remainder                                       VSYNC NOT ACCURATE
                None    => {}                                                                                   //else pass                                             VSYNC NOT ACCURATE
            }
        }
        
        if input.update(event) {                                                                                //handle input events on loop? not just on event
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {                                      //if esc pressed
                *control_flow = ControlFlow::Exit;                                                              //exit
                return;
            }

            if input.key_held(VirtualKeyCode::Up) {
                player_coords.1+=5;
            }
            if input.key_held(VirtualKeyCode::Down) {
                player_coords.1-=5;
            }
            if input.key_held(VirtualKeyCode::Left) {
                player_coords.0-=5;
            }
            if input.key_held(VirtualKeyCode::Right) {
                player_coords.0+=5;
            }
            if input.key_pressed(VirtualKeyCode::F3) {                                                          //if f3 pressed
                debug_flag = !debug_flag;                                                                       //toggle debug
            }

            if let Some(factor) = input.scale_factor_changed() {                                                //if window dimensions changed
                hidpi_factor = factor;                                                                          //update hidpi_factor
            }

            if let Some(size) = input.window_resized() {                                                        //if window resized
                pixels.resize(size.width, size.height);                                                         //resize pixel aspect ratio
            }

            //do world updates
            update_camera(&mut camera_coords, player_coords);                                                   //move camera towards player
            window.request_redraw();                                                                            //request frame redraw
        }
    });
}



///updates camera position based off player coords
fn update_camera(camera_coords: &mut (isize,isize), player_coords: (isize, isize)) {
    let distance_x = player_coords.0 - camera_coords.0;         //calc x coord distance
    let distance_y = player_coords.1 - camera_coords.1;         //calc y coord distance
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
fn draw_screen(world: &Vec<Vec<gen::Chunk>>, player_coords: (isize, isize), camera_coords: (isize, isize), debug_flag: bool, fps: usize, seed: &String) -> Vec<Vec<[u8;4]>> {
    let mut screen = drawing::draw_visible(world,camera_coords);                                                //gets visible pixels from world as 2d vec
    drawing::draw_debug_block(&mut screen, camera_coords, camera_coords, 5, [255;4]);                           //render camera
    drawing::draw_debug_block(&mut screen, player_coords, camera_coords, 5, [0;4]);                             //render player
    if ENABLE_DEBUG && debug_flag {drawing::draw_debug_screen(&mut screen, player_coords, camera_coords, fps, seed)}  //if debug flag and debug enabled: render debug
    drawing::draw_text(&mut screen, (20,SCREEN_HEIGHT-30), GAME_TITLE, 16.0, [255,255,255,0], DEBUG_FONT);      //render game title                         
    screen                                                                                                      //return 1d screen
}



//Chunk
//    coords: (i32,i32),
//    data: Vec<Vec<Particle>>
//         coords: (u8,u8)
//         data: Particle
//             rgba: [u8;4]

//gen_chunk = chunk index in vec
//world_chunk = world coordinates of chunk

#[test]
fn test_it(){}