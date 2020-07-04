use super::{gen::Chunk, player::Player};

use rusttype::{point, Font, Scale};

pub const DEBUG_FONT: &[u8; 19836] = include_bytes!("fonts/debug.ttf");

///draws current frame from 2D vec
pub fn flatten(screen: Vec<Vec<[u8;4]>>, frame: &mut [u8], screen_width: usize) {                
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {   //for spot in frame
        let pixel_x = i % screen_width;
        let pixel_y = i / screen_width;
        pixel.copy_from_slice(&screen[pixel_y][pixel_x]);       //put pixel at spot
    }
}



///maps all visible pixels to 2D vector from 4D vector
pub fn draw_visible(world: &Vec<Vec<Chunk>>, camera_coords: (isize, isize), screen_width: usize, screen_height: usize, chunk_width: usize, chunk_height: usize) -> Vec<Vec<[u8;4]>> {
    let mut screen = vec!(vec!([0;4]; screen_width); screen_height);                                                //creates black 2d vec for screen
    for gen_chunk_y in world{                                                                                       //for chunk layer
        for gen_chunk_x in gen_chunk_y{                                                                             //for chunk in layer
            for (local_y_coord, local_y) in gen_chunk_x.data.iter().enumerate(){                                    //for local layer in chunk
                for (local_x_coord, local_x) in local_y.iter().enumerate(){                                         //for Particle in local layer
                    let world_coords = get_world_coords(gen_chunk_x.chunk_coords, (local_x_coord, local_y_coord), chunk_width, chunk_height);  //get world coordinates from 0,0
                    let (pixel_x, pixel_y) = check_visible(world_coords, camera_coords, screen_width, screen_height);                            //check if pixel visible from camera
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



///temp function that draws block at target
pub fn draw_debug_block(screen: &mut Vec<Vec<[u8;4]>>, obj_coords: (isize, isize), camera_coords: (isize, isize), size: isize, color: [u8;4], screen_width: usize, screen_height: usize) {
    let screen_x = (screen_width/2) as isize - (camera_coords.0-obj_coords.0);          //calc obj x distance from camera
    let screen_y = (screen_height/2) as isize - (obj_coords.1-camera_coords.1);         //calc obj y distance from camera
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



///draws debug text
pub fn draw_debug_screen(screen: &mut Vec<Vec<[u8;4]>>, player: &mut Player, camera_coords: (isize,isize), fps: usize, seed: &String, chunk_width: usize) {
    draw_text(screen, (20,20), "DEBUG", 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("{} FPS", fps);
    draw_text(screen, (20,30), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Player: {}, {}", player.coords.0, player.coords.1);
    draw_text(screen, (20,40), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Velcoity: {:2.3}, {:2.3}", player.velocity.0, player.velocity.1);
    draw_text(screen, (20,50), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Chunk: {}, {} in {}, {}", player.coords.0 % chunk_width as isize, player.coords.1 % chunk_width as isize, 
                                            player.coords.0 / chunk_width as isize, player.coords.1 / chunk_width as isize,);
    draw_text(screen, (20,60), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Camera: {}, {}", camera_coords.0, camera_coords.1);
    draw_text(screen, (20,70), &s, 16.0, [255,255,255,0], DEBUG_FONT);
    let s = format!("Seed: {}", seed);
    draw_text(screen, (20,80), &s, 16.0, [255,255,255,0], DEBUG_FONT);
}



///draws raw text to screen
pub fn draw_text(screen: &mut Vec<Vec<[u8;4]>>, coords: (usize, usize), text: &str, font_size: f32, color: [u8;4], font_data: &[u8]) {
    let font = Font::try_from_bytes(font_data).expect("Error constructing Font");           //get font from DEBUG_FONT
    let scale = Scale::uniform(font_size);                                                  //calc font scale
    let v_metrics = font.v_metrics(scale);                                                  //calc font size
    let glyphs: Vec<_> = font                                                               //layout the glyphs in a line with 0 pixels padding
        .layout(text, scale, point(0.0, 0.0 + v_metrics.ascent))
        .collect();
    for glyph in glyphs {                                                                   //loop through the glyphs in the text, positing each one on a line
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {                                                          //draw the glyph into the image per-pixel by using the draw closure
                let px = x as usize + bounding_box.min.x as usize + coords.0;               //offset the position by the glyph bounding box
                let py = y as usize + bounding_box.min.y as usize + coords.1;
                if v != 0.0 {                                                               //turn the coverage into an alpha value
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



///checks if target pixel is visible based off:\
///    world coords, camera coords, and screen dimensions
fn check_visible(world_coords: (isize, isize), camera_coords: (isize,isize), screen_width: usize, screen_height: usize) -> (usize,usize){
    let distance_from_camera = {                                                                //gets (x,y) pixels +/- distance from camera
        let distance_x = world_coords.0 - camera_coords.0;                                      //calculates x
        let distance_y = world_coords.1 - camera_coords.1;                                      //calculates y
        (distance_x,distance_y)
    };                                                                                          //else if visible
    let calc_position = |distance: isize, length: isize| {                                      //closure that calcs target pixels position on screen 
        if distance > 0 {(distance+length/2) as usize}                                          //if + coord from cam add 1/2 screen len                                
        else {(length/2 - distance*-1) as usize}                                                //if - coord from cam make positive and subtract from half screen len
    };
    let pixel_x = calc_position(distance_from_camera.0, screen_width as isize);                 //calc x coord on screen
    let tmp_y = calc_position(distance_from_camera.1, screen_height as isize);
    let pixel_y = if tmp_y < screen_height {screen_height - tmp_y} else {tmp_y};                //calc y coord on screen                                                NEEDS screen_height- BUT NOT 100% SURE WHY TBH              
    (pixel_x,pixel_y)                                                                           //return position on screen
}



///calculates world coordinates based off chunk and local coords
fn get_world_coords(world_chunk_coords: (isize, isize), world_local_coords: (usize,usize), chunk_width: usize, chunk_height: usize) -> (isize, isize) {
    let wx = world_chunk_coords.0*chunk_width as isize+world_local_coords.0 as isize;       //calculates x
    let wy = world_chunk_coords.1*chunk_height as isize+world_local_coords.1 as isize*-1;   //calculates y                                                                  I HAVE NO IDEA WHY BUT *-1 FIXED Y AXIS CHUNK FLIP BUG
    (wx,wy)
}