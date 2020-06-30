use std::{thread, time};
use rand::Rng;


use log::error;
use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: usize = 768;
const SCREEN_HEIGHT: usize = 768;
const CHUNK_WIDTH: usize = 256;
const CHUNK_HEIGHT: usize = 256;
const GEN_RANGE: isize = 4;
const TARGET_FPS: u64 = 60;


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
    let mut camera_coords: (isize, isize) = (128,-128);                                                             //set camera location

    event_loop.run(move |event, _, control_flow| {                                                              //start game loop
        let frame_start = clock_ticks::precise_time_s();                                                      //get current loop start time
        if let Event::RedrawRequested(_) = event {                                                              //if redraw requested
            draw(get_screen(&world, camera_coords), pixels.get_frame());                                        //get screen then render screen
            if pixels                                                                                           //if rendering error
                .render()                                                                                                       
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err() {
                *control_flow = ControlFlow::Exit;                                                              //break
                return;
            }                
            frames+=1;                                                                                          //inc this seconds frame counter

            print!("\x1B[2J\x1B[1;1H{} FPS",fps);                                                               //print debug
            if (clock_ticks::precise_time_s() - fps_time) >= 1.0 {                                              //if second has passed since last second
                fps = frames;                                                                                   //fps = this seconds frames
                fps_time = clock_ticks::precise_time_s();                                                       //reset second time
                frames = 0;                                                                                     //reset second frames
            }
            
            match (frame_time).checked_sub(((clock_ticks::precise_time_s() - frame_start) * 1000000.0) as u64) {//if frame took less than target fps time
                Some(i) => {thread::sleep(time::Duration::from_micros(i))}                                      //sleep remainder
                None    => {}                                                                                   //else pass
            }
        }
        
        if input.update(event) {                                                                                //handle input events on loop? not just on event
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {                                      //if esc pressed
                *control_flow = ControlFlow::Exit;                                                              //exit
                return;
            }

            if input.key_held(VirtualKeyCode::Up) {
                camera_coords.1+=5;
            }
            if input.key_held(VirtualKeyCode::Down) {
                camera_coords.1-=5;
            }
            if input.key_held(VirtualKeyCode::Left) {
                camera_coords.0-=5;
            }
            if input.key_held(VirtualKeyCode::Right) {
                camera_coords.0+=5;
            }

            if let Some(factor) = input.scale_factor_changed() {                                                //if window dimensions changed
                hidpi_factor = factor;                                                                          //update hidpi_factor
            }

            if let Some(size) = input.window_resized() {                                                        //if window resized
                pixels.resize(size.width, size.height);                                                         //resize pixel aspect ratio
            }

            //do world updates
            window.request_redraw();                                                                            //request frame redraw
        }
    });
}


struct Chunk {                      //world chunk object
    chunk_coords: (isize,isize),    //chunk coordinates
    data: Vec<Vec<Cell>>,           //chunk cell data
}

impl Chunk {
    fn gen_chunk(chunk_coords: (isize,isize)) -> Self{                                          //generates new chunk with random color
        let mut data = vec![vec![Cell::new((0,0), [0;4]); CHUNK_WIDTH]; CHUNK_HEIGHT];          //generate black chunk
        let mut rng = rand::thread_rng();                                                       //get rng handle
        let rgba = [rng.gen(),rng.gen(),rng.gen(),0];                                           //generate random color values
        for y in 0..data.len() {                                                                //for y in data vec
            for x in 0..data[y].len() {                                                         //for x in y
                data[y][x] = Cell::new((x as u8, y as u8),rgba);                                //update color
            }
        }
        //BLACK BOX
        for y in 0..CHUNK_HEIGHT/25 {                                                           //creates little black box to show upper left of chunk
            for x in 0..CHUNK_WIDTH/25 {
                data[y][x].data.rgba = [0;4];
            }
        } 
        Self{                                                                                   //return instance of chunk
            chunk_coords,
            data
        }
    }
}


#[derive(Clone)]
struct Cell {                   //chunk cell data
    local_coords: (u8,u8),      //coordinates within chunk
    data: Particle,             //cell particle data
}

impl Cell {
    fn new(local_coords: (u8,u8), rgba: [u8; 4]) -> Self {  //generates a new cell with specified particle color
        Self {
            local_coords,
            data: Particle::new(rgba),
        }
    }
}


#[derive(Clone)]
struct Particle {   //cell particle data
    rgba: [u8;4]    //rgba color code
}

impl Particle {
    fn new(rgba: [u8;4]) -> Self {  //generate new particle
        Self {
            rgba
        }
    }
}



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



//draws current frame
fn draw(screen: Vec<[u8;4]>, frame: &mut [u8]) {                
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {   //for spot in frame
        pixel.copy_from_slice(&screen[i]);                      //put pixel at spot
    }
}



//gets current frame to draw
fn get_screen(world: &Vec<Vec<Chunk>>, camera_coords: (isize, isize)) -> Vec<[u8;4]> {
    let screen = get_visible(world,camera_coords);                  //gets visible pixels from world as 2d vec
    let mut screen_1d = vec!([0;4]; SCREEN_WIDTH*SCREEN_HEIGHT);    //creates black 1d vec
    let mut i = 0;                                                  //pixel index counter                           
    for pixel_y in screen {                                         //for y layer in visible pixels
        for pixel_x in pixel_y {                                    //for x in y layer
            screen_1d[i] = pixel_x;                                 //map to the id pixel index
            i+=1;                                                   //inc index
        }
    }
    screen_1d                                                       //return 1d screen
}



fn get_visible(world: &Vec<Vec<Chunk>>, camera_coords: (isize, isize)) -> Vec<Vec<[u8;4]>> {
    let mut screen = vec!(vec!([0;4]; SCREEN_WIDTH); SCREEN_HEIGHT);                                    //creates black 2d vec for screen
    for gen_chunk_y in world{                                                                           //for chunk layer
        for gen_chunk_x in gen_chunk_y{                                                                 //for chunk in layer
            for local_y in &gen_chunk_x.data{                                                           //for local layer in chunk
                for local_x in local_y{                                                                 //for cell in local layer
                    let world_coords = get_world_coords(gen_chunk_x.chunk_coords, local_x.local_coords);//get world coordinates from 0,0
                    match check_visible(world_coords, camera_coords){                                   //check if pixel visible from camera
                        Some((pixel_x,pixel_y)) => screen[pixel_y][pixel_x] = local_x.data.rgba,        //if visible place pixel on screen
                        None => ()
                    } 
                }
            }
        }
    }
    screen
}
 

//calculates world coordinates based off chunk and local coords
fn get_world_coords(world_chunk_coords: (isize, isize), world_local_coords: (u8,u8)) -> (isize, isize) {
    let wx = world_chunk_coords.0*CHUNK_WIDTH as isize+world_local_coords.0 as isize;       //calculates x
    let wy = world_chunk_coords.1*CHUNK_HEIGHT as isize+world_local_coords.1 as isize*-1;   //calculates y                                                                  I HAVE NO IDEA WHY BUT *-1 FIXED Y AXIS CHUNK FLIP BUG
    (wx,wy)
}


//DOESNT RETURN + EDGE PIXELS?
//returns pixels location on screen. returns None if pixel not visible
fn check_visible(world_coords: (isize, isize), camera_coords: (isize,isize)) -> Option<(usize,usize)>{
    let distance_from_camera = {                                                                    //gets (x,y) pixels +/- distance from camera
        let distance_x = world_coords.0 - camera_coords.0;                                          //calculates x
        let distance_y = world_coords.1 - camera_coords.1;                                          //calculates y
        (distance_x,distance_y)
    };
    let make_positive = |num| if num < 0 {(num*-1) as usize} else {num as usize};                   //closure that makes distance positive value for visibility check
    if make_positive(distance_from_camera.0) >= SCREEN_WIDTH/2                                      //if farther from camera than 1/2 of screen                             = REMOVES LEFT PIXEL COLUMN TO MAKE BUG LOOK LIKE FEATURE
    || make_positive(distance_from_camera.1) >= SCREEN_HEIGHT/2 {None}                              //return not visible                                                    = REMOVES TOP PIXEL ROW TO MAKE BUG LOOK LIKE FEATURE
    else {                                                                                          //else if visible
        let calc_position = |distance: isize, length: isize| {                                      //closure that calcs target pixels position on screen 
            if distance > 0 {(distance+length/2) as usize-1}                                        //if + coord from cam add 1/2 screen len                                -1 REQUIRED TO PREVENT CRASH BUT REMOVES BOTTOM ROW OF PIXELS
            else {(length/2 - distance*-1) as usize}                                                //if - coord from cam make positive and subtract from half screen len
        };
        let pixel_x = calc_position(distance_from_camera.0, SCREEN_WIDTH as isize);                 //calc x coord on screen
        let pixel_y = SCREEN_HEIGHT-calc_position(distance_from_camera.1, SCREEN_HEIGHT as isize)-1;//calc y coord on screen                                                -1 REQUIRED TO PREVENT CRASH BUT REMOVES RIGHT PIXEL COLUMN         NEEDS SCREEN_HEIGHT- AND -1 BUT NOT 100% SURE WHY TBH              
        Some((pixel_x,pixel_y))                                                                     //return position on screen
    }
}


//Chunk
//    coords: (i32,i32),
//    data: Vec<Vec<Cell>>
//         coords: (u8,u8)
//         data: Particle
//             rgba: [u8;4]

#[test]
fn test_it(){
    let world = init_world();
    let wc = get_world_coords(world[1][0].chunk_coords,world[0][1].data[50][56].local_coords);
    assert_eq!(wc, (0,0))
}