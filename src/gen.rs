use rand::{Rng, SeedableRng, rngs::StdRng};
use rand::distributions::Alphanumeric;
use noise::{NoiseFn, Perlin, Seedable};


pub type World = Vec<Vec<Chunk>>;


///contains chunk data
pub struct Chunk {                      //world chunk object
    pub chunk_coords: (isize,isize),    //chunk coordinates
    pub data: Vec<Vec<Particle>>,       //chunk Particle data
}

impl Chunk {
    ///generates a random colored chunk\
    ///that contains a 2d vector
    fn _gen_test(chunk_coords: (isize,isize), rng: &mut StdRng, chunk_width: usize, chunk_height: usize) -> Self{            //generates new chunk with random color
        let mut data = vec![vec![Particle::new([0;4]); chunk_width]; chunk_height]; //generate black chunk
        let rgba = [rng.gen(),rng.gen(),rng.gen(),0];                               //generate random color values
        for y in 0..data.len() {                                                    //for y in data vec
            for x in 0..data[y].len() {                                             //for x in y
                data[y][x] = Particle::new(rgba);                                   //update color
            }
        }
        //BLACK BOX
        for y in 0..chunk_height/25 {                                               //creates little black box to show upper left of chunk
            for x in 0..chunk_width/25 {
                data[y][x].rgba = [0;4];
            }
        }
        Self{                                                                       //return instance of chunk
            chunk_coords,
            data
        }
    }

    fn gen_perlin(chunk_coords: (isize, isize), generator: Perlin, chunk_width: usize, chunk_height: usize) -> Self {
        let mut data = vec!(vec!(Particle::new([0;4]); chunk_width); chunk_height);
        for y in 0..data.len() {
            for x in 0..data[y].len() {
                let perlx = ((chunk_coords.0 * chunk_width as isize) as f64 + x as f64)/500.0;
                let perly = ((chunk_coords.1 * chunk_height as isize) as f64 + y as f64 * -1.0)/500.0;      //WHY DOES THIS NEED *-1?? WITHOUT IT IT FLIPS EVERY CHUNKS Y AXIS
                //let rgba = 255.0 * ((generator.get([perlx,perly])+1.0)/2.0);
                let rgba = 255.0 * generator.get([perlx,perly]);
                data[y][x] = Particle::new([rgba as u8;4]);
            }
        }
        //BLACK BOX
        for y in 0..chunk_height/25 {                                               //creates little black box to show upper left of chunk
            for x in 0..chunk_width/25 {
                data[y][x].rgba = [0;4];
            }
        }
        Self {
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



///generates starting area\
///whats inside is temporary
pub fn _init_test_world(rng: &mut StdRng, gen_range: isize, chunk_width: usize, chunk_height: usize) -> World {
    let mut world: World = Vec::new();                                                //create empty world
    for (yi, world_chunk_y) in (gen_range*-1..gen_range+1).rev().enumerate() {                                    //for chunk layer coordinate in gen range
        world.push(Vec::new());                                                                 //push new layer to vec
        for world_chunk_x in gen_range*-1..gen_range+1 {                                        //for chunk x_pos in gen range
            world[yi].push(Chunk::_gen_test((world_chunk_x, world_chunk_y), rng, chunk_width, chunk_height));  //generate chunk and push to layer
        }                                                             
    }
    world                                                                                       //return newly generated world
}

pub fn init_perlin_world(generator: Perlin, gen_range: isize, chunk_width: usize, chunk_height: usize) -> World {
    let mut world= Vec::new();
    for (yi, world_chunk_y) in (gen_range*-1..gen_range+1).rev().enumerate() {
        world.push(Vec::new());
        for world_chunk_x in gen_range*-1..gen_range+1 {
            world[yi].push(Chunk::gen_perlin((world_chunk_x, world_chunk_y), generator, chunk_width, chunk_height));
        }
    }
    world
}



pub fn get_screen(screen: &mut Vec<Vec<[u8;4]>>, world: &World, camera_coords: (isize, isize), screen_width: usize, screen_height: usize, chunk_width: usize, chunk_height: usize) {
    let camera = get_local_coords(world, camera_coords, chunk_width, chunk_height);                             //gets coords of camera in loaded chunks
    for (py, y) in (camera.1 - screen_height as isize/2..camera.1 + screen_height as isize/2).enumerate() {     //for screen pixel index and particle in range of camera y
        for (px, x) in (camera.0 - screen_width as isize/2..camera.0 + screen_width as isize/2).enumerate() {   //for screen pixel index and particle in range of camera x
            let ((cx,cy),(lx,ly)) = get_local_coord_pair((y as usize,x as usize), chunk_width, chunk_height);   //get chunk xy ald inner xy from local xy
            if let Some(c_row) = world.get(cy) {                                                                //attempt to get chunk row
                if let Some(c) = c_row.get(cx) {                                                                //attempt to get chunk in row
                    screen[py][px] = c.data[ly][lx].rgba;                                                       //copy color of target particle in chunk
                } else {screen[py][px] = [0;4]}                                                                 //if target chunk doesn't exist color black
            } else {screen[py][px] = [0;4]}                                                                     //if target chunk row doesn't exist color black
        }
    }
}

///calculates chunk (x,y) and internal (x,y) from local coordinates
fn get_local_coord_pair(coords: (usize, usize), chunk_width: usize, chunk_height: usize) -> ((usize, usize),(usize, usize)) {
    ((coords.0/chunk_width, coords.1/chunk_height),(coords.0%chunk_width, coords.1%chunk_height))
}

///calculates local coordinates in world vec from your global position
///returns negative if above/left of rendered area
fn get_local_coords(world: &World, coords: (isize, isize), chunk_width: usize, chunk_height: usize) -> (isize, isize) {
    let (wx, wy) = world[0][0].chunk_coords;            //gets coords of first chunk in rendered vec
    let lx = coords.0 - (wx * chunk_width as isize);    //calculates local x coord based off world coords of first chunk
    let ly = (wy * chunk_height as isize) - coords.1;   //calculates local y coord based off world coords of first chunk
    (lx, ly)
}

///handle seeding of world
pub fn _get_rng_test(set_seed: bool, seed: &str) -> (StdRng, String) {
    let mut full_seed = seed.to_string();                                               //set full_seed as supplied seed
    let mut display_seed = seed.to_string();                                            //set seed to display as supplied seed
    if !set_seed {                                                                      //if set seed flag not set
        full_seed = rand::thread_rng().sample_iter(&Alphanumeric).take(32).collect();   //generate new 32 char seed
        display_seed = full_seed.clone();                                               //set display seed to new seed
    }
    let mut bytes_seed = [0;32];                                                        //create 32 byte placeholder for seed byte array
    for i in 0..bytes_seed.len() {                                                      //for index in byte_seed
        bytes_seed[i] = match full_seed.as_bytes().get(i) {                             //let byte_seed index = try get full_seed byte at index
            Some(byte) => *byte,                                                        //if valid index return byte
            None => 0,                                                                  //else return 0
        }
    }
    let rng: StdRng = SeedableRng::from_seed(bytes_seed);                               //set world rng seed
    (rng, display_seed)                                                                 //return handle to world rng and seed to be displayed
}

pub fn get_perlin_generator(set_seed: bool, mut seed: u32) -> Perlin {
    if !set_seed {                                                                      //if set seed flag not set
        seed = rand::thread_rng().gen();
    }
    Perlin::new().set_seed(seed)
}
