use rand::Rng;
use noise::{NoiseFn, Perlin, Seedable};


pub struct World {
    pub data: Vec<Vec<Chunk>>,
    _generator: Perlin,
    pub seed: u32,
    _gen_range: isize,
}

impl World {
    pub fn new_perlin(chunk_dim: (usize, usize), seed: &mut u32, set_seed: bool, gen_range: isize) -> Self{
        if !set_seed {                                                                                  //if set seed flag not set
            *seed = rand::thread_rng().gen();                                                           //gen random seed
        }
        let generator = Perlin::new().set_seed(*seed);                                                  //return Perlin generator

        let mut data= Vec::new();                                                                       //creates empty vec to hold world
        for (yi, world_chunk_y) in (gen_range*-1..gen_range+1).rev().enumerate() {                      //for y index, y in gen range counting down
            data.push(Vec::new());                                                                      //push new row
            for world_chunk_x in gen_range*-1..gen_range+1 {                                            //for chunk in gen range of row
                data[yi].push(Chunk::gen_perlin((world_chunk_x, world_chunk_y), generator, chunk_dim)); //gen new perlin chunk and put it there
            }
        }

        Self {
            data,
            _generator: generator,
            seed: *seed,
            _gen_range: gen_range,
        }
    }

    ///calculates local coordinates in world vec from your global position
    ///returns negative if above/left of rendered area
    pub fn get_local_coords(&self, coords: (isize, isize), chunk_dim: (usize, usize)) -> (isize, isize) {
        let (wx, wy) = self.data[0][0].chunk_coords;    //gets coords of first chunk in rendered vec
        let lx = coords.0 - (wx * chunk_dim.0 as isize);//calculates local x coord based off world coords of first chunk
        let ly = (wy * chunk_dim.1 as isize) - coords.1;//calculates local y coord based off world coords of first chunk
        (lx, ly)
    }

    pub fn get_local_pair(coord: isize, chunk: usize) -> (usize, usize) {
        (coord as usize/chunk, coord as usize%chunk)
    }

    ///gets all visible pixels on screen relative camera position in world
    #[inline]
    pub fn get_screen(&self, screen: &mut Vec<u8>, camera_coords: (isize, isize), screen_dim: (usize, usize), chunk_dim: (usize, usize)) {
        let camera = self.get_local_coords(camera_coords, chunk_dim);                       //gets loaded coords of camera in loaded chunks
        (camera.1..camera.1 + screen_dim.1 as isize).enumerate().for_each(|(py,y)| {        //for screen pixel index and particle in range of camera loaded y
            let (cy, ly) = World::get_local_pair(y, chunk_dim.1);                           //calculate chunk y and inner y from loaded y
            if let Some(c_row) = self.data.get(cy) {                                        //if chunk row at loaded chunk y exists
                (camera.0..camera.0 + screen_dim.0 as isize).enumerate().for_each(|(px,x)| {//for screen pixel index and particle in range of camera loaded x
                    let (cx,lx) = World::get_local_pair(x, chunk_dim.0);                    //get loaded chunk x and inner x from loaded x
                    let pos = (py*screen_dim.0+px)*4;
                    if let Some(c) = c_row.get(cx) {screen[pos..pos+4].copy_from_slice(&c.data[ly][lx].rgba[..])}  //if chunk in row then copy color of target particle in chunk
                    else {screen[pos..pos+4].copy_from_slice(&[0;4][..])}                                           //if target chunk doesn't exist color black
                })
            } else {screen[(py*screen_dim.0)*4..(py*screen_dim.0+screen_dim.0)*4].chunks_exact_mut(4).for_each(|px| px.copy_from_slice(&[0;4][..]))}                       //if target chunk row doesn't exist color row black
        });
    }

    pub fn check_collision(&self, hitbox: Vec<(isize, isize)>, chunk_dim: (usize, usize)) -> bool {
        for (wx,wy) in hitbox {                                     //for coord pair in hitbox vec
            let (lx,ly) = self.get_local_coords((wx,wy), chunk_dim);//get local coords
            let (lcx, inx) = World::get_local_pair(lx, chunk_dim.0);//get loaded x chunk and internal x
            let (lcy, iny) = World::get_local_pair(ly, chunk_dim.1);//get loaded y chunk and internal y
            if let Some(c_row) = self.data.get(lcy) {               //if row valid
                if let Some(c) = c_row.get(lcx) {                   //and column valid
                    if c.data[iny][inx].collision {                 //if x,y has collision
                       return true                                  //return true
                    }
                }
            }
        }
        false                                                       //if none had collision return false
    }
}



///contains chunk data
#[derive(Clone)]
pub struct Chunk {                      //world chunk object
    pub chunk_coords: (isize,isize),    //chunk coordinates
    pub data: Vec<Vec<Particle>>,       //chunk Particle data
}

impl Chunk {
    ///generates chunk using perlin noise
    fn gen_perlin(chunk_coords: (isize, isize), generator: Perlin, chunk_dim: (usize, usize)) -> Self {
        let mut data = vec!(vec!(Particle::new([0;4], false); chunk_dim.0); chunk_dim.1);               //creates empty vec for particles
        for y in 0..data.len() {                                                                        //for row in len of chunk

            let gen_depth = {                                                                           //generates number based off depth that slowly climb's from -1 up to 0.1
                let mut depth = (((chunk_coords.1*chunk_dim.1 as isize) as f64-y as f64)/1000.0)*-1.0;  //  this makes it so caves don't generate at the surface
                depth = depth - 1.0;
                if depth > 0.1 {
                    0.1
                } else {depth}
            };

            for x in 0..data[y].len() {                                                                 //for particle in row
                let perlx = ((chunk_coords.0 * chunk_dim.0 as isize) as f64 + x as f64)/500.0;          //set coords for perlin noise
                let perly = ((chunk_coords.1 * chunk_dim.1 as isize) as f64 - y as f64)/500.0;
                let noise = generator.get([perlx,perly]);                                               //get noise for chunk

                let mut ground = 0;                                                                     //sets ground level default to 0 (always above)
                if chunk_coords.1 == 0 {                                                                //if coord is at 0 ground level
                    ground = (255.0 * ((noise+1.0)/2.0)) as usize;                                      //convert noise to 0-255 to resemble ground level
                } else if chunk_coords.1 > 0 {ground = 256}                                             //else if chunk below ground set ground level to 256 (always below)

                if y >= ground {                                                                        //if y below ground level
                    let particle = {                                                                    //create particle:
                        if noise > gen_depth {Particle::new([124, 94, 66, 255], true)}                  //if noise great enough to gen caves at current depth level return cave particle
                        else {Particle::new([135, 206, 235, 0], false)}                                 //else return sky particle
                    };                                                                                  //map noise to shades of grey
                    data[y][x] = particle;                                                              //copy color to particle in chunk x,y
                } else {data[y][x] = Particle::new([135, 206, 235, 0], false)}
            }
        }
        //BLACK BOX
        //for y in 0..chunk_height/25 {                                                                 //creates little black box to show upper left of chunk
        //    for x in 0..chunk_width/25 {
        //        data[y][x].rgba = [0;4];
        //    }
        //}
        Self {                                                                                          //return chunk
            chunk_coords,
            data,
        }
    }
}



///contains all particle data
#[derive(Clone)]
pub struct Particle {   //Particle particle data
    pub rgba: [u8;4],   //rgba color code
    pub collision: bool,//if it has collision
}

impl Particle {
    ///creates a colored particle
    fn new(rgba: [u8;4], collision: bool) -> Self {  //generate new particle
        Self {
            rgba,
            collision,
        }
    }
}