pub trait Rng {
    fn gen(&mut self) -> u64;
}

#[derive(Debug, Clone, Copy)]
pub struct Splitmix64 {
    state: u64
}

impl Splitmix64 {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub const fn gen_const(self) -> (Self, u64) {
        let state = self.state.wrapping_add(0x9E3779B97f4A7C15);
        let mut result = state;

        result = (result ^ result.wrapping_shr(30)).wrapping_mul(0xBF58476D1CE4E5B9);
        result = (result ^ result.wrapping_shr(37)).wrapping_mul(0x94D049BB133111EB);

        result = result ^ result.wrapping_shr(31);

        (Self { state }, result)
    }
}

impl Rng for Splitmix64 {
    fn gen(&mut self) -> u64 {
        let (state, result) = self.gen_const();
        *self = state;

        result
    }
}


/// A simple Xoshiro 256** pRNG
#[derive(Debug, Clone, Copy)]
pub struct Xoshiro256ss {
    state: [u64; 4]
}


impl Xoshiro256ss {
    pub const fn new(seed: u64) -> Self {
        let mut seed = Splitmix64::new(seed);
        let mut init = Self { state: [0; 4] };

        // unroll the loop for const ..
        let (next_seed, tmp) = seed.gen_const();
        init.state[0] = tmp;
        seed = next_seed;
        let (next_seed, tmp) = seed.gen_const();
        init.state[1] = tmp;
        seed = next_seed;
        let (next_seed, tmp) = seed.gen_const();
        init.state[2] = tmp;
        seed = next_seed;
        let (_, tmp) = seed.gen_const();
        init.state[3] = tmp;

        init
    }
}


impl Rng for Xoshiro256ss {
    fn gen(&mut self) -> u64 {
        let res = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.state[1].wrapping_shl(17);

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        res
    }
}
