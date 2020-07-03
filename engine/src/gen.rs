use rand::{Rng, SeedableRng, rngs::StdRng};
use rand::distributions::Alphanumeric;

///contains chunk data
pub struct Chunk {                      //world chunk object
    pub chunk_coords: (isize,isize),    //chunk coordinates
    pub data: Vec<Vec<Particle>>,       //chunk Particle data
}

impl Chunk {
    ///generates a random colored chunk\
    ///that contains a 2d vector
    fn gen_chunk(chunk_coords: (isize,isize), rng: &mut StdRng, chunk_width: usize, chunk_height: usize) -> Self{            //generates new chunk with random color
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
}



///cotains all particle data
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
pub fn init_world(rng: &mut StdRng, gen_range: isize, chunk_width: usize, chunk_height: usize) -> Vec<Vec<Chunk>> {                                         
    let mut world: Vec<Vec<Chunk>> = Vec::new();                                                //create empty world
    let mut loaded_chunk_y = 0;                                                                 //create y index counter
    for world_chunk_y in (gen_range*-1..gen_range+1).rev() {                                    //for chunk layer coordinate in gen range 
        world.push(Vec::new());                                                                 //push new layer to vec
        for world_chunk_x in gen_range*-1..gen_range+1 {                                        //for chunk x_pos in gen range
            world[loaded_chunk_y].push(Chunk::gen_chunk((world_chunk_x, world_chunk_y), rng, chunk_width, chunk_height));  //generate chunk and push to layer
        }
        loaded_chunk_y+=1;                                                                      //inc y layer
    }
    world                                                                                       //return newly generated world
}



///handle seeding of world
pub fn get_rng(set_seed: bool, seed: &str) -> (StdRng, String) {
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