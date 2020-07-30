use rusttype::{point, Font, Scale};
use image::{png, ImageDecoder, AnimationDecoder, ImageError};
use image::gif::GifDecoder;
use std::fs::File;
use std::convert::TryInto;

pub const DEBUG_FONT: &[u8; 19836] = include_bytes!("fonts/debug.ttf");

pub type Screen = Vec<Vec<[u8;4]>>;

///draws current frame from 2D vec
pub fn flatten(screen: &Screen, frame: &mut [u8], screen_width: usize) {                
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {   //for spot in frame
        let pixel_x = i % screen_width;
        let pixel_y = i / screen_width;
        pixel.copy_from_slice(&screen[pixel_y][pixel_x]);       //put pixel at spot
    }
}




///temp function that draws block at target
pub fn draw_debug_block(screen: &mut Screen, obj_coords: (isize, isize), camera_coords: (isize, isize), size: isize, color: [u8;4], screen_width: usize, screen_height: usize) {
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



///draws raw text to screen
pub fn draw_text(screen: &mut Screen, coords: (usize, usize), text: &str, font_size: f32, color: [u8;4], font_data: &[u8]) {
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





///load spritesheet from frames of gif
pub fn load_spritesheet(path: &str) -> Result<Vec<Vec<Vec<[u8;4]>>>, ImageError> {
    let f = File::open(path)?;                                          //open spritesheet gif
    let decoder = GifDecoder::new(f).unwrap();                          //decode gif
    let frames = decoder.into_frames();                                 //get frames of gif
    let frames = frames.collect_frames()?;                              //collect frames

    let mut spritesheet: Vec<Vec<Vec<[u8;4]>>> = Vec::new();            //create vec for each frame
    for (i_img, frame) in frames.iter().enumerate() {                   //for index, frame in frames
        spritesheet.push(Vec::new());                                   //add vec to spritesheet
        let buf = frame.clone().into_buffer();                          //clone frame bytes into buffer
        let width = buf.width() as usize*4;                             //get width of sprite
        let buf = buf.into_vec();                                       //create vec to hold sprite data
        for (i_row,row) in buf.chunks(width).enumerate() {              //for index, row in buf
            spritesheet[i_img].push(Vec::new());                        //create new row vec in sprite data
            for p in row.chunks(4) {                                    //for pixel in row of bytes
                spritesheet[i_img][i_row].push(p.try_into().unwrap())   //push pixel to row
            }
        }
    } 
    Ok(spritesheet)                                                     //return spritesheet
}



///loads sprite from png
pub fn load_sprite(path: &str) -> Result<Vec<Vec<[u8;4]>>, ImageError> {
    let f = File::open(path)?;                              //open sprite file
    let decoder = png::PngDecoder::new(f)?;                 //create decoder from sprite file
    let width = decoder.dimensions().0 as usize*4;          //get sprite byte width
    let mut buf = vec!(0;decoder.total_bytes() as usize);   //create buf to hold sprite data
    decoder.read_image(&mut buf)?;                          //read sprite data into buf
    
    let mut img: Vec<Vec<[u8;4]>> = Vec::new();             //create img to hold sprite data
    for (i, row) in buf.chunks(width).enumerate(){          //for row of bytes in buf 
        img.push(Vec::new());                               //create new vec
        for p in row.chunks(4) {                            //for pixel in row of bytes
            img[i].push(p.try_into().unwrap());             //convert bytes to pixel and push
        }
    }

    Ok(img)                                                 //return sprite
}



///scale spritesheet
pub fn scale_spritesheet(spritesheet: &Vec<Vec<Vec<[u8;4]>>>, scale: usize) -> Vec<Vec<Vec<[u8;4]>>> {
    let mut scaled_spritesheet = Vec::new();                //create var to hold scaled sprites
    for frame in spritesheet {                              //for each frame in spritesheet
        scaled_spritesheet.push(scale_sprite(frame, scale));//scale sprite
    }       
    scaled_spritesheet                                      //return scaled spritesheet
}



///scales sprite
pub fn scale_sprite(sprite: &Vec<Vec<[u8;4]>>, scale: usize) -> Vec<Vec<[u8;4]>>{
    let mut scaled_sprite = Vec::new();                 //create scaled_sprite to hold sprite data
    for row in sprite {                                 //for each row in sprite
        for _ in 0..scale {                             //for scale factor times
            scaled_sprite.push(Vec::new());             //push a new row to scaled_sprite
            let i = scaled_sprite.len()-1;              //get index of new row in scaled_sprite
            for x in row {                              //for pixel in row of sprite
                for _ in 0..scale {                     //for scale factor times
                    scaled_sprite[i].push(x.clone());   //push pixel to new row
                }
            }
        }
    }
    scaled_sprite                                       //return scaled_sprite
}



///draws sprite to screen, ignoring transparent pixels
pub fn draw_sprite(screen: &mut Vec<Vec<[u8;4]>>, sprite: &Vec<Vec<[u8;4]>>, coords: (usize, usize)) {
    for (yi, row) in sprite.iter().enumerate() {                            //for row index, row
        for (xi, pixel) in row.iter().enumerate() {                         //for column index, pixel
            if pixel[3] != 0 {                                              //if pixel not transparent
                if let Some(scr_row) = screen.get_mut(coords.1+yi) {        //check if row exists on screen
                    if let Some(scr_pixel) = scr_row.get_mut(coords.0+xi) { //check if column exists on screen
                        *scr_pixel = pixel.clone();                         //if yes push pixel to location
                    }
                }
            }
        }
    }
}