use std::{thread, time};
use rand::Rng;
use rusttype::{point, Font, Scale};

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
const CHUNK_WIDTH: usize = 256;
const CHUNK_HEIGHT: usize = 256;
const GEN_RANGE: isize = 4;
const TARGET_FPS: u64 = 70;    //VSYNC NOT ACCURATE

const GAME_TITLE: &str = "Untitled Game v0.001";
const ENABLE_DEBUG: bool = true;
const DEBUG_FONT: &[u8; 19836] = include_bytes!("fonts/debug.ttf");


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
            .with_title("Prototype Chunk Generation")
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

    let world = init_world();                                                                                   //generate world
    let mut player_coords: (isize, isize) = (128,-128);                                                         //set player location
    let mut camera_coords: (isize, isize) = (128,-128);                                                         //set camera location
    let mut debug_flag = false;

    event_loop.run(move |event, _, control_flow| {                                                              //start game loop
        let frame_start = clock_ticks::precise_time_s();                                                        //get current loop start time                           VSYNC NOT ACCURATE
        if let Event::RedrawRequested(_) = event {                                                              //if redraw requested
            draw(render_screen(&world, player_coords, camera_coords, debug_flag, fps), pixels.get_frame());     //get screen then render screen
            if pixels                                                                                           //if rendering error
                .render()                                                                                                       
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err() {
                *control_flow = ControlFlow::Exit;                                                              //break
                return;
            }                
            frames+=1;                                                                                          //inc this seconds frame counter

            print!("\x1B[2J\x1B[1;1H{} FPS\nPLAYER: {:?}\nCAMERA {:?}",fps, player_coords, camera_coords);      //print debug
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



///contains chunk data
struct Chunk {                      //world chunk object
    chunk_coords: (isize,isize),    //chunk coordinates
    data: Vec<Vec<Particle>>,       //chunk Particle data
}

impl Chunk {
    ///generates a random colored chunk\
    ///that contains a 2d vector
    fn gen_chunk(chunk_coords: (isize,isize)) -> Self{                              //generates new chunk with random color
        let mut data = vec![vec![Particle::new([0;4]); CHUNK_WIDTH]; CHUNK_HEIGHT]; //generate black chunk
        let mut rng = rand::thread_rng();                                           //get rng handle
        let rgba = [rng.gen(),rng.gen(),rng.gen(),0];                               //generate random color values
        for y in 0..data.len() {                                                    //for y in data vec
            for x in 0..data[y].len() {                                             //for x in y
                data[y][x] = Particle::new(rgba);                                   //update color
            }
        }
        //BLACK BOX
        for y in 0..CHUNK_HEIGHT/25 {                                               //creates little black box to show upper left of chunk
            for x in 0..CHUNK_WIDTH/25 {
                data[y][x].rgba = [0;4];
            }
        } 
        Self{                                                                       //return instance of chunk
            chunk_coords,
            data
        }
    }
}



///cotains all particle data
#[derive(Clone)]
struct Particle {   //Particle particle data
    rgba: [u8;4]    //rgba color code
}

impl Particle {
    ///creates a colored particle
    fn new(rgba: [u8;4]) -> Self {  //generate new particle
        Self {
            rgba
        }
    }
}


///generates starting area\
///whats inside is temporary
fn init_world() -> Vec<Vec<Chunk>> {                                                        //initalizes world
    let mut world: Vec<Vec<Chunk>> = Vec::new();                                            //create empty world
    let mut loaded_chunk_y = 0;                                                             //create y index counter
    for world_chunk_y in (GEN_RANGE*-1..GEN_RANGE+1).rev() {                                //for chunk layer coordinate in gen range 
        world.push(Vec::new());                                                             //push new layer to vec
        for world_chunk_x in GEN_RANGE*-1..GEN_RANGE+1 {                                    //for chunk x_pos in gen range
            world[loaded_chunk_y].push(Chunk::gen_chunk((world_chunk_x, world_chunk_y)));   //generate chunk and push to layer
        }
        loaded_chunk_y+=1;                                                                  //inc y layer
    }
    world                                                                                   //return newly generated world
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



///draws current frame from 1D vec
fn draw(screen: Vec<[u8;4]>, frame: &mut [u8]) {                
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {   //for spot in frame
        pixel.copy_from_slice(&screen[i]);                      //put pixel at spot
    }
}


///temp function that renders block at target
fn render_block(screen: &mut Vec<Vec<[u8;4]>>, obj_coords: (isize, isize), camera_coords: (isize, isize), size: isize, color: [u8;4]) {
    let screen_x = (SCREEN_WIDTH/2) as isize - (camera_coords.0-obj_coords.0);          //calc obj x distance from camera
    let screen_y = (SCREEN_HEIGHT/2) as isize - (obj_coords.1-camera_coords.1);         //calc obj y distance from camera
    for y in screen_y-size..screen_y+size {                                             //for pixel in y range
        for x in screen_x-size..screen_x+size {                                         //for pixel in x range
            match screen.get(y as usize) {                                              //attempt y index
                Some(py) => match py.get(x as usize) {                                  //if valid y index attempt x index
                    Some(_) => screen[y as usize][x as usize] = color,                  //if valid x index draw pixel
                    None => ()
                },
                None => ()
            }
        }
    } 
    
}



///draws raw text to screen
pub fn draw_text(screen: &mut Vec<Vec<[u8;4]>>, coords: (usize, usize), text: &str, font_size: f32, color: [u8;4], font_data: &[u8]) {
    let font = Font::try_from_bytes(font_data).expect("Error constructing Font");           //get font from DEBUG_FONT
    let scale = Scale::uniform(font_size);                                                  //calc font scale
    let v_metrics = font.v_metrics(scale);                                                  //calc font size
    let glyphs: Vec<_> = font                                                               // layout the glyphs in a line with 0 pixels padding
        .layout(text, scale, point(0.0, 0.0 + v_metrics.ascent))
        .collect();
    for glyph in glyphs {                                                                   // Loop through the glyphs in the text, positing each one on a line
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {                                                          // Draw the glyph into the image per-pixel by using the draw closure
                let px = x as usize + bounding_box.min.x as usize + coords.0;               // Offset the position by the glyph bounding box
                let py = y as usize + bounding_box.min.y as usize + coords.1;
                if v != 0.0 {                                                               // Turn the coverage into an alpha value
                    match screen.get(py) {                                                  //attempt y index
                        Some(pyt) => match pyt.get(px) {                                    //attempt x index
                            Some(_) => screen[py][px] =color,                               //if valid x,y draw pixel
                            _ => (),
                        },
                        _ => (),
                    }
                }
            });
        }
    }
}



///renders debug text
fn render_debug(screen: &mut Vec<Vec<[u8;4]>>, player_coords: (isize,isize), camera_coords: (isize,isize), fps: usize) {
    draw_text(screen, (20,20), "DEBUG", 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("{} FPS", fps);
    draw_text(screen, (20,30), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Player: {}, {}", player_coords.0, player_coords.1);
    draw_text(screen, (20,40), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Chunk: {}, {} in {}, {}", player_coords.0 % CHUNK_WIDTH as isize, player_coords.1 % CHUNK_WIDTH as isize, player_coords.0 / CHUNK_WIDTH as isize, player_coords.1 / CHUNK_WIDTH as isize,);
    draw_text(screen, (20,50), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Camera: {}, {}", camera_coords.0, camera_coords.1);
    draw_text(screen, (20,60), &s, 16.0, [255,255,255,0], DEBUG_FONT);
}




///gets 1D vec of current frame to draw from 4D Vec
fn render_screen(world: &Vec<Vec<Chunk>>, player_coords: (isize, isize), camera_coords: (isize, isize), debug_flag: bool, fps: usize) -> Vec<[u8;4]> {
    let mut screen = get_visible(world,camera_coords);                                              //gets visible pixels from world as 2d vec
    render_block(&mut screen, camera_coords, camera_coords, 5, [255;4]);                            //render camera
    render_block(&mut screen, player_coords, camera_coords, 5, [0;4]);                              //render player
    if ENABLE_DEBUG && debug_flag {render_debug(&mut screen, player_coords, camera_coords, fps)}    //if debug flag and debug enabled: render debug
    draw_text(&mut screen, (20,SCREEN_HEIGHT-30), GAME_TITLE, 16.0, [255,255,255,0], DEBUG_FONT);   //render game title
    let mut screen_1d = vec!([0;4]; SCREEN_WIDTH*SCREEN_HEIGHT);                                    //creates black 1d vec
    let mut i = 0;                                                                                  //pixel index counter                           
    for pixel_y in screen {                                                                         //for y layer in visible pixels
        for pixel_x in pixel_y {                                                                    //for x in y layer
            screen_1d[i] = pixel_x;                                                                 //map to the id pixel index
            i+=1;                                                                                   //inc index
        }
    }
    screen_1d                                                                                       //return 1d screen
}





///maps all visible pixels to 2D vector from 4D vector
fn get_visible(world: &Vec<Vec<Chunk>>, camera_coords: (isize, isize)) -> Vec<Vec<[u8;4]>> {
    let mut screen = vec!(vec!([0;4]; SCREEN_WIDTH); SCREEN_HEIGHT);                                                //creates black 2d vec for screen
    for gen_chunk_y in world{                                                                                       //for chunk layer
        for gen_chunk_x in gen_chunk_y{                                                                             //for chunk in layer
            for (local_y_coord, local_y) in gen_chunk_x.data.iter().enumerate(){                                    //for local layer in chunk
                for (local_x_coord, local_x) in local_y.iter().enumerate(){                                         //for Particle in local layer
                    let world_coords = get_world_coords(gen_chunk_x.chunk_coords, (local_x_coord, local_y_coord));  //get world coordinates from 0,0
                    let (pixel_x, pixel_y) = check_visible(world_coords, camera_coords);                            //check if pixel visible from camera
                    match screen.get(pixel_y) {                                                                     //attempt y index
                        Some(py) => match py.get(pixel_x) {                                                         //if valid y index attempt x index
                            Some(_) => screen[pixel_y][pixel_x] =local_x.rgba,                                      //if valid x,y map pixel
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
        }
    }
    screen
}
 

///calculates world coordinates based off chunk and local coords
fn get_world_coords(world_chunk_coords: (isize, isize), world_local_coords: (usize,usize)) -> (isize, isize) {
    let wx = world_chunk_coords.0*CHUNK_WIDTH as isize+world_local_coords.0 as isize;       //calculates x
    let wy = world_chunk_coords.1*CHUNK_HEIGHT as isize+world_local_coords.1 as isize*-1;   //calculates y                                                                  I HAVE NO IDEA WHY BUT *-1 FIXED Y AXIS CHUNK FLIP BUG
    (wx,wy)
}




///checks if target pixel is visible based off:\
///    world coords, camera coords, and screen dimensions
fn check_visible(world_coords: (isize, isize), camera_coords: (isize,isize)) -> (usize,usize){
    let distance_from_camera = {                                                                //gets (x,y) pixels +/- distance from camera
        let distance_x = world_coords.0 - camera_coords.0;                                      //calculates x
        let distance_y = world_coords.1 - camera_coords.1;                                      //calculates y
        (distance_x,distance_y)
    };                                                                                          //else if visible
    let calc_position = |distance: isize, length: isize| {                                      //closure that calcs target pixels position on screen 
        if distance > 0 {(distance+length/2) as usize}                                          //if + coord from cam add 1/2 screen len                                
        else {(length/2 - distance*-1) as usize}                                                //if - coord from cam make positive and subtract from half screen len
    };
    let pixel_x = calc_position(distance_from_camera.0, SCREEN_WIDTH as isize);                 //calc x coord on screen
    let tmp_y = calc_position(distance_from_camera.1, SCREEN_HEIGHT as isize);
    let pixel_y = if tmp_y < SCREEN_HEIGHT {SCREEN_HEIGHT - tmp_y} else {tmp_y};                //calc y coord on screen                                                NEEDS SCREEN_HEIGHT- BUT NOT 100% SURE WHY TBH              
    (pixel_x,pixel_y)                                                                           //return position on screen
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
fn test_it(){


}