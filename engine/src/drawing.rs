use rusttype::{point, Font, Scale};

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



