//use std::{thread, time};
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
const RENDER_RANGE: isize = 1;
//const TARGET_FPS: u64 = 120;


fn main() {
    let mut frames = 0;                                                                                         //frame counter
    let mut fps_time = clock_ticks::precise_time_s();                                                           //keeps track of when a second passes
    let mut fps = 0;                                                                                            //stores last seconds fps
    //let frame_time = 1000000 / TARGET_FPS;                                                                      //target fps

    let event_loop = EventLoop::new();                                                                          //create event loop obj
    let mut input = WinitInputHelper::new();                                                                    //create WinitIH obj
    let window = {                                                                                              //create window obj
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
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
    let camera_coords: (isize, isize) = (0,0);                                                                  //set camera location

    event_loop.run(move |event, _, control_flow| {                                                              //start game loop
        //let frame_start = clock_ticks::precise_time_s();                                                        //get current loop start time
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
            
            //match (frame_time).checked_sub(((clock_ticks::precise_time_s() - frame_start) * 1000000.0) as u64) {//if frame took less than target fps time
            //    Some(i) => {thread::sleep(time::Duration::from_micros(i))}                                      //sleep remainder
            //    None    => {}                                                                                   //else pass
            //}
        }
        
        if input.update(event) {                                                                                //handle input events on loop? not just on event
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {                                      //if esc pressed
                *control_flow = ControlFlow::Exit;                                                              //exit
                return;
            }

            if let Some(factor) = input.scale_factor_changed() {                                                //if window dimensions changed
                hidpi_factor = factor;                                                                          //update hidpi_factor
            }

            if let Some(size) = input.window_resized() {                                                        //if window resized
                pixels.resize(size.width, size.height);                                                         //resize pixel aspect ratio
            }

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
        let mut data = vec![vec![Cell::new((0,0), [0,0,0,0]); CHUNK_WIDTH]; CHUNK_HEIGHT];      //generate black chunk
        let mut rng = rand::thread_rng();                                                       //get rng handle
        let rgba = [rng.gen(),rng.gen(),rng.gen(),0];                                           //generate random color values
        for y in 0..data.len() {                                                                //for y in data vec
            for x in 0..data[y].len() {                                                         //for x in y
                data[y][x] = Cell::new((x as u8, y as u8),rgba);                                //update color
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



fn init_world() -> Vec<Vec<Chunk>> {                    //initalizes world
    let mut world: Vec<Vec<Chunk>> = Vec::new();        //create empty world
    let mut y = 0;                                      //create y index counter
    for cy in (RENDER_RANGE*-1..RENDER_RANGE+1).rev() { //for chunk layer coordinate in render range 
        world.push(Vec::new());                         //push new layer to vec
        for cx in RENDER_RANGE*-1..RENDER_RANGE+1 {     //for chunk x_pos in render range
            world[y].push(Chunk::gen_chunk((cx, cy)));  //generate chunk and push to layer
        }
        y+=1;                                           //inc y layer
    }
    world                                               //return newly generated world
}



fn draw(screen: Vec<[u8;4]>, frame: &mut [u8]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        pixel.copy_from_slice(&screen[i]);
    }
}



fn get_screen(world: &Vec<Vec<Chunk>>, camera_coords: (isize, isize)) -> Vec<[u8;4]> {
    let screen = get_visible(world,camera_coords);
    let mut screen_1d = vec!([0;4]; SCREEN_WIDTH*SCREEN_HEIGHT);
    let mut i = 0;
    for y in screen {
        for x in y {
            screen_1d[i] = x;
            i+=1;
        }
    }
    screen_1d
}




fn get_visible(world: &Vec<Vec<Chunk>>, camera_coords: (isize, isize)) -> Vec<Vec<[u8;4]>> {
    let mut screen = vec!(vec!([0;4]; SCREEN_WIDTH); SCREEN_HEIGHT);
    let mut pixel = 0;
    for celly in 0..world.len()*CHUNK_HEIGHT{                       //0-3
        let ly = celly % CHUNK_HEIGHT;
        let cy = celly / CHUNK_HEIGHT;
        for cellx in 0..world[cy].len()*CHUNK_WIDTH {  //0-768
            let lx = cellx % CHUNK_WIDTH;
            let cx = cellx / CHUNK_WIDTH;
            let wc = get_world_coords(world[cy][cx].chunk_coords, (lx as u8,ly as u8));
            if check_visible(wc, camera_coords){
                let px = pixel % SCREEN_WIDTH;
                let py = pixel / SCREEN_WIDTH;
                screen[py][px] = world[cy][cx].data[ly][lx].data.rgba;
                pixel+=1;
                //print!("{}\r",pixel);
            }
        }
    }
    screen
}


fn check_visible(wc: (isize, isize), camera_coords: (isize,isize)) -> bool{
    let distance_from_camera = {        
        let mut dx = wc.0 - camera_coords.0; if dx < 0 {dx = dx*-1}
        let mut dy = wc.0 - camera_coords.1; if dy < 0 {dy = dy*-1}
        (dx as usize,dy as usize)
    };
    if distance_from_camera.0 < SCREEN_WIDTH/2
    && distance_from_camera.1 < SCREEN_HEIGHT/2 {true}
    else {false}
}

fn get_world_coords(chunk_coords: (isize, isize), local_coords: (u8,u8)) -> (isize, isize) {
    let wx = chunk_coords.0*CHUNK_WIDTH as isize+local_coords.0 as isize;
    let wy = chunk_coords.1* CHUNK_HEIGHT as isize+local_coords.1 as isize;
    (wx,wy)
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