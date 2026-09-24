use rand::{Rng, RngExt, SeedableRng, rngs::StdRng, seq::IndexedRandom};

pub struct GameRng {
    rng: StdRng,
}

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    pub fn next_u8(&mut self) -> u8 {
        self.rng.random()
    }

    pub fn random_bool(&mut self, probability: f64) -> bool {
        self.rng.random_bool(probability)
    }

    pub fn u8_below(&mut self, upper: u8) -> u8 {
        self.rng.random_range(0..upper)
    }

    pub fn u8_inclusive(&mut self, upper: u8) -> u8 {
        self.rng.random_range(0..=upper)
    }

    pub fn choose<'a, T>(&mut self, values: &'a [T]) -> Option<&'a T> {
        values.choose(&mut self.rng)
    }

    pub fn sample<'a, T>(&mut self, values: &'a [T], amount: usize) -> Vec<&'a T> {
        values.sample(&mut self.rng, amount).collect()
    }
}
