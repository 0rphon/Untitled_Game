use rand::Rng;
use noise::{NoiseFn, Perlin, Seedable};


pub type World = Vec<Vec<Chunk>>;


///contains chunk data
pub struct Chunk {                      //world chunk object
    pub chunk_coords: (isize,isize),    //chunk coordinates
    pub data: Vec<Vec<Particle>>,       //chunk Particle data
}

impl Chunk {
    ///generates chunk using perlin noise
    fn gen_perlin(chunk_coords: (isize, isize), generator: Perlin, chunk_width: usize, chunk_height: usize) -> Self {
        let mut data = vec!(vec!(Particle::new([0;4]); chunk_width); chunk_height);                 //creates empty vec for particles
        for y in 0..data.len() {                                                                    //for row in len of chunk
            for x in 0..data[y].len() {                                                             //for particle in row
                let perlx = ((chunk_coords.0 * chunk_width as isize) as f64 + x as f64)/500.0;      //calculate perlin x coord
                let perly = ((chunk_coords.1 * chunk_height as isize) as f64 + y as f64*-1.0)/500.0;//calculate perlin y coord              WHY DOES THIS NEED *-1?? WITHOUT IT IT FLIPS EVERY CHUNKS Y AXIS
                //let rgba = 255.0 * ((generator.get([perlx,perly])+1.0)/2.0);
                let rgba = 255.0 * generator.get([perlx,perly]);                                    //map noise to shades of grey
                data[y][x] = Particle::new([rgba as u8;4]);                                         //copy color to particle in chunk x,y
            }
        }
        //BLACK BOX
        for y in 0..chunk_height/25 {                                                               //creates little black box to show upper left of chunk
            for x in 0..chunk_width/25 {
                data[y][x].rgba = [0;4];
            }
        }
        Self {                                                                                      //return chunk
            chunk_coords,
            data,
        }
    }
}



///contains all particle data
#[derive(Clone)]
pub struct Particle {   //Particle particle data
    pub rgba: [u8;4]    //rgba color code
}

impl Particle {
    ///creates a colored particle
    fn new(rgba: [u8;4]) -> Self {  //generate new particle
        Self {
            rgba
        }
    }
}



///generates a world of perlin noise
pub fn init_perlin_world(generator: Perlin, gen_range: isize, chunk_width: usize, chunk_height: usize) -> World {
    let mut world= Vec::new();                                                                                      //creates empty vec to hold world
    for (yi, world_chunk_y) in (gen_range*-1..gen_range+1).rev().enumerate() {                                      //for y index, y in gen range counting down
        world.push(Vec::new());                                                                                     //push new row
        for world_chunk_x in gen_range*-1..gen_range+1 {                                                            //for chunk in gen range of row
            world[yi].push(Chunk::gen_perlin((world_chunk_x, world_chunk_y), generator, chunk_width, chunk_height));//gen new perlin chunk and put it there
        }
    }
    world
}

///gets all visible pixels on screen relative camera position in world
#[inline]
pub fn get_screen(screen: &mut Vec<Vec<[u8;4]>>, world: &World, camera_coords: (isize, isize), screen_width: usize, screen_height: usize, chunk_width: usize, chunk_height: usize) {
    let camera = get_local_coords(world, camera_coords, chunk_width, chunk_height);                                 //gets loaded coords of camera in loaded chunks
    (camera.1 - screen_height as isize/2..camera.1 + screen_height as isize/2).enumerate().for_each(|(py,y)| {      //for screen pixel index and particle in range of camera loaded y
        let (cy, ly) = (y as usize/chunk_height, y as usize%chunk_height);                                          //calculate chunk y and inner y from loaded y
        if let Some(c_row) = world.get(cy) {                                                                        //if chunk row at loaded chunk y exists
            (camera.0 - screen_width as isize/2..camera.0 + screen_width as isize/2).enumerate().for_each(|(px,x)| {//for screen pixel index and particle in range of camera loaded x
                let (cx,lx) = (x as usize/chunk_width, x as usize%chunk_width);                                     //get loaded chunk x and inner x from loaded x
                if let Some(c) = c_row.get(cx) {                                                                    //attempt to get chunk in row
                    screen[py][px] = c.data[ly][lx].rgba;                                                           //copy color of target particle in chunk
                } else {screen[py][px] = [0;4]}                                                                     //if target chunk doesn't exist color black
            })      
        } else {screen[py].iter_mut().for_each(|px| *px = [0;4])}                                                   //if target chunk row doesn't exist color row black
    });
}

///calculates local coordinates in world vec from your global position
///returns negative if above/left of rendered area
pub fn get_local_coords(world: &World, coords: (isize, isize), chunk_width: usize, chunk_height: usize) -> (isize, isize) {
    let (wx, wy) = world[0][0].chunk_coords;            //gets coords of first chunk in rendered vec
    let lx = coords.0 - (wx * chunk_width as isize);    //calculates local x coord based off world coords of first chunk
    let ly = (wy * chunk_height as isize) - coords.1;   //calculates local y coord based off world coords of first chunk
    (lx, ly)
}



///gets handle to perlin noise generator
pub fn get_perlin_generator(set_seed: bool, mut seed: u32) -> Perlin {
    if !set_seed {                          //if set seed flag not set
        seed = rand::thread_rng().gen();    //gen random seed
    }
    Perlin::new().set_seed(seed)            //return Perlin generator
}


