mod player;
mod gen;
use engine::{drawing, game};

const SCREEN_HEIGHT: usize = 1080;//528;
const SCREEN_WIDTH: usize = 1920;//960;
//const ASPECT_RATIO: f32 = 9.0/16.0;
//const SCREEN_WIDTH: usize = (SCREEN_HEIGHT as f32 / ASPECT_RATIO)as usize;
const TARGET_FPS: u64 = 60;

const GAME_TITLE: &str = "Untitled Game v0.002";
const ENABLE_DEBUG: bool = true;        //if debug can be toggled
const DO_BENCHMARKS: bool = false;

const CHUNK_WIDTH: usize = 256;
const CHUNK_HEIGHT: usize = 256;
const GEN_RANGE: isize = 25;             //how far out to gen chunks
const SET_SEED: bool = true;           //if seed should be set
const SEED: u32 = 8675309;



fn main() {
    if DO_BENCHMARKS {
        do_tests();
    }


    let generator = gen::get_perlin_generator(SET_SEED, SEED);                                                              //get rng and display_seed
    let world = gen::init_perlin_world(generator, GEN_RANGE, CHUNK_WIDTH, CHUNK_HEIGHT);                                    //generate world
    let mut screen: drawing::Screen = vec!(vec!([0;4]; SCREEN_WIDTH); SCREEN_HEIGHT);                                       //create blank screen buffer
    let mut player = player::Player::spawn((0,0), drawing::load_sprite("sprites/dude.png").unwrap());                       //spawn player at 0,0
    let mut camera_coords: (isize, isize) = (0,0);                                                                          //set camera location
    let mut debug_flag = false;

    let mut fpslock = game::FpsLock::create_lock(TARGET_FPS);                                                               //create fps lock obj

    let event_loop = game::EventLoop::new();                                                                                //create event loop obj
    let mut input = game::WinitInputHelper::new();                                                                          //create input helper obj
    let mut window = game::Window::init(GAME_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT, &event_loop);                              //create window, and pixels buffer


    event_loop.run(move |event, _, control_flow| {                                                                          //start game loop
        fpslock.start_frame();                                                                                              //start frame for fps lock
        if let game::Event::RedrawRequested(_) = event {                                                                    //if redraw requested
            draw_screen(&mut screen, &world, &mut player, camera_coords, debug_flag, fpslock.get_fps(), SEED);              //draws new frame to screen buffer
            drawing::flatten(&screen, window.pixels.get_frame());                                                           //flatten screen to 1D for render
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
            
            if let Some(factor) = input.scale_factor_changed() {                                                            //if window dimensions changed
                window.hidpi_factor = factor;                                                                               //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                                    //if window resized
                window.pixels.resize(size.width, size.height);                                                              //resize pixel aspect ratio
            }

            do_updates(&mut camera_coords, &mut player);
            window.window.request_redraw();                                                                                 //request frame redraw
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
fn draw_screen(screen: &mut drawing::Screen, world: &Vec<Vec<gen::Chunk>>, player: &mut player::Player, camera_coords: (isize, isize), debug_flag: bool, fps: usize, seed: u32) {
    gen::get_screen(screen, world,camera_coords, SCREEN_WIDTH, SCREEN_HEIGHT, CHUNK_WIDTH, CHUNK_HEIGHT);                                   //gets visible pixels from world as 2d vec
    drawing::draw_sprite(screen, &player.sprite, drawing::get_screen_coords(player.coords, camera_coords, SCREEN_WIDTH, SCREEN_HEIGHT));    //draw player sprite
    if ENABLE_DEBUG && debug_flag {                                                                                                         //if debug flag and debug enabled:
        drawing::draw_debug_block(screen,                                                                                                   //render debug block on camera
                                    drawing::get_screen_coords(camera_coords, 
                                                            camera_coords, 
                                                            SCREEN_WIDTH, 
                                                            SCREEN_HEIGHT), 
                                                            5, 
                                                            [255;4]);   
        drawing::draw_debug_box(screen,                                                                                                     //render debug outline on player
                                    drawing::get_screen_coords(player.coords, 
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
pub fn draw_debug_screen(screen: &mut drawing::Screen, player: &mut player::Player, camera_coords: (isize,isize), fps: usize, seed: u32, chunk_width: usize) {
    drawing::draw_text(screen, (20,20), "DEBUG", 16.0, [255,0,0,0], drawing::DEBUG_FONT);
    let s = format!("{} FPS", fps);
    drawing::draw_text(screen, (20,30), &s, 16.0, [255,0,0,0], drawing::DEBUG_FONT);
    let s = format!("Player: {}, {}", player.coords.0, player.coords.1);
    drawing::draw_text(screen, (20,40), &s, 16.0, [255,0,0,0], drawing::DEBUG_FONT);
    let s = format!("Velocity: {:2.3}, {:2.3}", player.velocity.0, player.velocity.1);
    drawing::draw_text(screen, (20,50), &s, 16.0, [255,0,0,0], drawing::DEBUG_FONT);
    let s = format!("Chunk: {}, {} in {}, {}", player.coords.0 % chunk_width as isize, player.coords.1 % chunk_width as isize, 
                                            player.coords.0 / chunk_width as isize, player.coords.1 / chunk_width as isize,);
    drawing::draw_text(screen, (20,60), &s, 16.0, [255,0,0,0], drawing::DEBUG_FONT);
    let s = format!("Camera: {}, {}", camera_coords.0, camera_coords.1);
    drawing::draw_text(screen, (20,70), &s, 16.0, [255,0,0,0], drawing::DEBUG_FONT);
    let s = format!("Seed: {}", seed);
    drawing::draw_text(screen, (20,80), &s, 16.0, [255,0,0,0], drawing::DEBUG_FONT);
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


fn do_tests() {
    println!("Doing tests");
    let batch_size = 1_000;
    old_test(batch_size);
    new_test(batch_size);
    panic!("ok");
}

fn old_test(batch_size: u32) {
    use std::time::Instant;

    let generator = gen::get_perlin_generator(SET_SEED, SEED);                                                              //get rng and display_seed
    let world = gen::init_perlin_world(generator, GEN_RANGE, CHUNK_WIDTH, CHUNK_HEIGHT);                                    //generate world
    let mut screen = vec!(vec!([0;4]; SCREEN_WIDTH); SCREEN_HEIGHT);                                    //create blank screen buffer
    let start = Instant::now();
    for _ in 0..batch_size {                                                                                //start loop
        gen::get_screen(&mut screen, &world,(0,0) , SCREEN_WIDTH, SCREEN_HEIGHT, CHUNK_WIDTH, CHUNK_HEIGHT);                                   //gets visible pixels from world as 2d vec
    }
    println!("old: {:?} {:?}", start.elapsed(), start.elapsed()/batch_size);
}

fn new_test(batch_size: u32) {
    println!("new: {:?} ", batch_size);
}