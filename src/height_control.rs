use skiplist::SkipList;

use std;
use std::default::Default;

extern crate rand;
use self::rand::{random, Open01};

pub trait HeightControl<K> {
    fn max_height(&self) -> usize;
    fn get_height(&mut self, key: &K) -> usize;
}

pub struct GeometricalGenerator {
    upgrade_probability_: f64,
    max_height_: usize,
}

impl GeometricalGenerator {
    pub fn new(max_height: usize, upgrade_probability: f64) -> GeometricalGenerator {
        GeometricalGenerator {
            upgrade_probability_: upgrade_probability,
            max_height_: max_height,
        }
    }
}

impl<K> HeightControl<K> for GeometricalGenerator {
    #[inline(always)]
    fn max_height(&self) -> usize {
        self.max_height_
    }

    #[allow(unused_variables)]
    fn get_height(&mut self, key: &K) -> usize {
        // Simulates a random variate with geometric distribution. The idea is
        // that we are modelling number of successes until the first failure.
        let mut h = 0;

        while h < self.max_height_ {
            let Open01(throw) = random::<Open01<f64>>();
            if throw >= self.upgrade_probability_ {
                return h;
            }

            h += 1;
        }

        h
    }
}

// 'HashCoinGenerator' creates heights by using a hash function that distributes
// uniformly among the output universe and counting the number of trailing zeros
// in the hashed value of a key. This is akin to using a Geometric(1/2) for all
// practical purposes; however, it is faster than generating a random number for
// an unknown amount of time, assuming that the hash function is not too
// expensive.
pub struct HashCoinGenerator<K, H> {
    max_height_: usize,
    hasher_: H,
    phantom_: std::marker::PhantomData<K>,
}

impl<K: std::hash::Hash, H: std::hash::Hasher> HashCoinGenerator<K, H> {
    pub fn new(max_height: usize, hasher: H) -> HashCoinGenerator<K, H> {
        HashCoinGenerator {
            max_height_: max_height,
            hasher_: hasher,
            phantom_: std::marker::PhantomData,
        }
    }
}

impl<K: std::hash::Hash, H: std::hash::Hasher> HeightControl<K> for HashCoinGenerator<K, H> {
    #[inline(always)]
    fn max_height(&self) -> usize {
        self.max_height_
    }

    fn get_height(&mut self, key: &K) -> usize {
        // We expect the hash function to be uniformly distributed over the
        // output universe. This means that the probability of getting a
        // sequence of trailing zeros of zero-based length i is (1/2)^(i + 1)
        key.hash(&mut self.hasher_);
        let height = self.hasher_.finish().trailing_zeros() as usize;
        // TODO: this is biased to low end values, unless max_height_ is a power
        // of two.
        let output = height % self.max_height_;
        output
    }
}

// 'HashingGenerator' creates heights by calling a hash function and capping the
// value at the maximum allowed.
pub struct HashingGenerator<K, H> {
    max_height_: usize,
    hasher_: H,
    phantom_: std::marker::PhantomData<K>,
}

impl<K: std::hash::Hash, H: std::hash::Hasher> HashingGenerator<K, H> {
    pub fn new(max_height: usize, hasher: H) -> HashingGenerator<K, H> {
        HashingGenerator {
            max_height_: max_height,
            hasher_: hasher,
            phantom_: std::marker::PhantomData,
        }
    }
}

impl<K: std::hash::Hash, H: std::hash::Hasher> HeightControl<K> for HashingGenerator<K, H> {
    #[inline(always)]
    fn max_height(&self) -> usize {
        self.max_height_
    }

    fn get_height(&mut self, key: &K) -> usize {
        key.hash(&mut self.hasher_);
        let height = self.hasher_.finish() as usize;
        // TODO: this is biased to low end values, unless max_height_ is a power
        // of two.
        let output = height % self.max_height_;
        output
    }
}

impl<K: 'static + std::hash::Hash + Default> Default for SkipList<K> {
    // TODO: fix when SipHasher is no longer deprecated
    #[allow(deprecated)]
    #[inline(always)]
    fn default() -> Self {
        let generator 
            = HashCoinGenerator::new(16, std::hash::SipHasher::default());
        Self::new(Box::new(generator))
    }
}