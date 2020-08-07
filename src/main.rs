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
const GEN_RANGE: isize = 25;             //how far out to gen chunks
const SET_SEED: bool = true;           //if seed should be set

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
    let mut camera_coords: (isize, isize) = (0,0);                                                                          //set camera location
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
fn draw_screen(screen: &mut drawing::Screen, world: &gen::World, player: &mut player::Player, camera_coords: (isize, isize), debug_flag: bool, fps: usize, seed: u32, mouse: &Mouse) {
    world.get_screen(&mut screen.buf, camera_coords, SCREEN_DIM, CHUNK_DIM);                            //gets visible pixels from world as 2d vec
    screen.draw_sprite(&player.sprite.get_sprite(), screen.get_coords(player.coords, camera_coords));   //draw player sprite
    if ENABLE_DEBUG && debug_flag {                                                                     //if debug flag and debug enabled:
        screen.draw_debug_block(screen.get_coords(camera_coords, camera_coords), 5, [255;4]);           //render debug block on camera
        screen.draw_debug_box(screen.get_coords(player.coords, camera_coords),                          //render debug outline on player
                                (player.sprite.get_sprite().width, 
                                player.sprite.get_sprite().height), 
                                [255,0,0,0]);   
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









//use engine::{drawing, game};
//use std::time::{Instant,Duration};
//
//const SCREEN_HEIGHT: usize = 1080;
//const SCREEN_WIDTH: usize = 1920;
//const TARGET_FPS: u64 = 60;
//const GAME_TITLE: &str = "Alex's Big Day";
//
//enum Facing {
//    Right,
//    Left,
//}
//
//
//#[derive(Clone)]
//struct Pixel {
//    collision: bool,
//}
//
//impl Pixel {
//    fn new_pixel(collision: bool) -> Self {
//        Self {
//            collision,
//        }
//    }
//}
//
//struct World {
//    data: Vec<Vec<Pixel>>,
//}
//
//impl World {
//    fn initialize(level: &Vec<World_Obj>) -> Self {
//        let mut world = vec!(vec!(Pixel::new_pixel(false);SCREEN_WIDTH);SCREEN_HEIGHT);
//
//        for obj in level {
//            if obj.collision {
//                let size = (obj.sprite.width as isize/2, obj.sprite.height as isize/2);
//                for yi in obj.obj_coords.1-size.1..obj.obj_coords.1+size.1 {
//                    if let Some(y) = world.get_mut(yi as usize) {
//                        for xi in obj.obj_coords.0-size.0..obj.obj_coords.0+size.0 {
//                            if let Some(x) = y.get_mut(xi as usize) {
//                                x.collision = true;
//                            }
//                        }
//                    }
//                }
//            }
//        }
//
//        Self{
//            data:world
//        }
//    }
//}
//
//struct Player_Obj {
//    sprite: drawing::Sprite,
//    player_coords: (isize,isize),
//    facing: Facing,
//    collision: bool
//}
//
//struct World_Obj {
//    sprite: drawing::Sprite,
//    obj_coords: (isize,isize),
//    collision: bool
//}
//
//impl Player_Obj {
//    fn alex() -> Self {
//        Self{
//            sprite: drawing::Sprite::load("sprites/alex.png").unwrap(),
//            player_coords: ((SCREEN_WIDTH as isize/2) + 20, (SCREEN_HEIGHT as isize/2) + 20),
//            facing: Facing::Left,
//            collision: true,
//            
//        }
//    }
//
//    fn check_collision(&self, world: &World, coords: (isize,isize)) -> bool {
//        if self.collision {                                                                  //if char has collision, runs this fn
//            let size = (self.sprite.width as isize/2, self.sprite.height as isize/2);        //takes half the size of sprite
//            for yi in coords.1-size.1..coords.1+size.1 {                                     //
//                if let Some(y) = world.data.get(yi as usize) {
//                    if let Some(x) = y.get((coords.0-size.0) as usize) {
//                        if x.collision {return true}
//                    }
//                    if let Some(x) = y.get((coords.0+size.0) as usize) {
//                        if x.collision {return true}
//
//                    }
//                }
//            }            
//            for xi in coords.0-size.0..coords.0+size.0 {
//                if let Some(y) = world.data.get((coords.0-size.0)as usize) {
//                    if let Some(x) = y.get(xi as usize) {
//                        if x.collision {return true}
//
//                    }
//                }
//                if let Some(y) = world.data.get((coords.0+size.0)as usize) {
//                    if let Some(x) = y.get(xi as usize) {
//                        if x.collision {return true}
//                    }
//                }
//            }
//        }
//        false
//    }
//    
//    fn try_move(&mut self, world: &World, coords: (isize,isize)) {
//        if !self.check_collision(world, coords) {
//            self.player_coords = coords;
//        } 
//    }
//}
//
//impl World_Obj {
//    fn basic_floor(coords: (isize,isize)) -> Self {
//        Self{
//            sprite: drawing::Sprite::load("sprites/floor_tile.png").unwrap(),
//            obj_coords: coords,
//            collision: true,
//         }
//    }
//}
//
//fn main() {
//    let level = vec!(
//        World_Obj::basic_floor((800,450))
//    );
//    
//    let world = World::initialize(&level);
//    
//    let mut alex = Player_Obj::alex();
// 
//    let mut mouse_coords = (0.0,0.0);                                                                   //creates empty tuple to hold mouse coords
//
//    let mut screen = drawing::Screen::new(SCREEN_WIDTH, SCREEN_HEIGHT);                                 //create blank screen buffer
//
//    let mut fpslock = game::FpsLock::create_lock(TARGET_FPS);                                           //create fpslock obj
//
//
//
//    //let event_loop = game::EventLoop::new();                                                          //create event loop obj
//    let event_loop = game::EventLoopExtWindows::new_any_thread();
//    let mut input = game::WinitInputHelper::new();                                                      //create input helper obj
//    let mut window = game::Window::init(GAME_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT, &event_loop);          //create window, and pixels buffer
//    window.fullscreen();                                                                                //make window fullscreen. one of the few winit features integrated into my lib
//    //info on window settings here https://docs.rs/winit/0.22.2/winit/window/struct.Window.html         //there are a ton of winit window settings but you'll need to import winit for most of them
//    window.window.set_cursor_grab(true).unwrap();                                                       //EXAMPLE binds cursor to window
//
//
//    event_loop.run(move |event, _, control_flow| {                                                      //start game loop
//        fpslock.start_frame();                                                                          //set start of frame time
//        if let game::Event::RedrawRequested(_) = event {                                                //if redraw requested
//
//            //MODIFY SCREEN HERE
//            screen.wipe();
//            
//            match alex.facing {
//                Facing::Right => screen.draw_sprite(&alex.sprite,alex.player_coords),
//                Facing::Left => screen.draw_sprite(&alex.sprite.get_flip(),alex.player_coords),
//            }     
//
//            for obj in &level {
//                screen.draw_sprite(&obj.sprite,obj.obj_coords);
//            }         
//            
//
//            screen.flatten(window.pixels.get_frame());                                                  //flatten screen to 1d Vec<[u8;4]>
//            window.pixels.render().unwrap();                                                            //render
//
//            fpslock.end_frame();
//        }
//
//        if input.update(event) {                                                                        //handle input events on loop? not just on event
//
//            //GET GAME INPUT HERE
//            //info on keys at https://docs.rs/winit/0.5.2/winit/enum.VirtualKeyCode.html
//            //info on events at https://docs.rs/winit_input_helper/0.7.0/winit_input_helper/struct.WinitInputHelper.html
//            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                        //if esc pressed
//                *control_flow = game::ControlFlow::Exit;                                                //exit
//                return;
//            }
//
//            if input.key_held(game::VirtualKeyCode::W) {println!("W")}                                  //EXAMPLE
//            if input.key_held(game::VirtualKeyCode::A) {
//                alex.try_move(&world, (alex.player_coords.0-5,alex.player_coords.1));
//                alex.facing = Facing::Left;
//            }                                  //EXAMPLE
//            if input.key_held(game::VirtualKeyCode::S) {alex.try_move(&world, (alex.player_coords.0,alex.player_coords.1+5))}                                  //EXAMPLE
//            if input.key_held(game::VirtualKeyCode::D) {
//                alex.try_move(&world, (alex.player_coords.0+5,alex.player_coords.1));
//                alex.facing = Facing::Right;
//            }                                  //EXAMPLE
//            if input.key_pressed(game::VirtualKeyCode::Space){alex.try_move(&world, (alex.player_coords.0,alex.player_coords.1-50))}                         //EXAMPLE
//            if input.key_pressed(game::VirtualKeyCode::LShift) {println!("Running")}                    //EXAMPLE
//            else if input.key_released(game::VirtualKeyCode::LShift){println!("Stopped running")}       //EXAMPLE
//            if input.key_pressed(game::VirtualKeyCode::F3) {println!("F3")}                             //EXAMPLE
//            if let Some(m) = input.mouse() {mouse_coords = m}                                           //EXAMPLE
//
//            if let Some(factor) = input.scale_factor_changed() {                                        //if window dimensions changed
//                window.hidpi_factor = factor;                                                           //update hidpi_factor
//            }
//            if let Some(size) = input.window_resized() {                                                //if window resized
//                window.pixels.resize(size.width, size.height);                                          //resize pixel aspect ratio
//            }
//
//            //DO WORLD UPDATES HERE
//            alex.try_move(&world, (alex.player_coords.0,alex.player_coords.1+5));
//
//            window.window.request_redraw();                                                             //request frame redraw
//        }
//    });
//}