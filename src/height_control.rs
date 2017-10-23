use skiplist::SkipList;

use std;
use std::default::Default;

extern crate rand;

/// This trait is for structures that implement a height generation strategy for
/// `SkipList<K>`.
///
/// Types that implement this trait are expected to be used as arguments to
/// `SkipList<K>::new`, and are responsible for returning the height to be used
/// for any given element that will be inserted in the Skip List.
///
/// Users should avoid implementing this trait unless there are effectively
/// space or speed concerns and they are certain that a change in the strategy
/// will fix their problem.
pub trait HeightControl<K> {
    /// Returns the maximum height that this controller can generate. This value
    /// must be constant per instance: implementors of this trait should not
    /// allow the output of this function to ever change after an instance has
    /// been created.
    ///
    /// # Remarks
    ///
    /// This is used by the Skip List to decide how much space to allocate for
    /// the head node. The impact of this value is very high: every search in
    /// the skip list needs to allocate a vector of the size given by this
    /// function. Searches happen in every action on it except for iteration.
    ///
    /// It is also called very frequently, which means that it should ideally be
    /// both inlineable and O(1).
    fn max_height(&self) -> usize;

    /// Generates a height for the `key`.
    ///
    /// # Arguments
    ///
    ///  * `key`: element for which the height should be generated.
    ///
    /// # Remarks
    ///
    /// The height returned by this function will be the level of the node for
    /// this `key`. This function does not need to be referentially transparent,
    /// and it may or may not be non-deterministic.
    ///
    /// This function highly influences the runtime of all operations in the
    /// SkipList. Exercise caution when implementing this function. A few things
    /// to keep in mind:
    ///
    ///  1. It is called in every insertion. Ideally, it should be O(1).
    ///  2. High values for this function mean more pointers between nodes and
    ///     bigger vectors.
    ///  3. Nodes of a given level will all be linked between themselves, so it
    ///     also affects the search strategy.
    ///  4. If you are using a RNG for this function, then that means you will
    ///     have to update the internal state whenever doing an insertion; try
    ///     to keep these updates within control.
    fn get_height(&mut self, key: &K) -> usize;
}

/// Implements height generation through simulation of a capped geometrical
/// random variable.
///
/// This implements the algorithm in the original paper:
/// * William Pugh. 1990. "Skip lists: a probabilistic alternative to balanced
///   trees". Commun. ACM 33, 6 (June 1990), 668-676.
///   DOI=http://dx.doi.org/10.1145/78973.78977
///
/// And is only included here for completeness, `PowTwoGenerator` should always
/// be preferred.
pub struct GeometricalGenerator {
    upgrade_probability_: f64,
    max_height_: usize,
}

impl GeometricalGenerator {
    /// Builds a new `GeometricalGenerator`
    ///
    /// # Arguments
    ///
    ///  * `max_height`: maximum height that the generator may give out to any
    ///    node.
    ///  * `upgrade_probability`: the probability used when simulating the
    ///    geometrical random variable.
    ///
    /// # Remarks
    ///
    /// This generator uses an RNG to simulate up to `max_heights` coin throws
    /// in every `get_height` call. This is slow, so it should be avoided.
    pub fn new(max_height: usize, upgrade_probability: f64) -> GeometricalGenerator {
        GeometricalGenerator {
            upgrade_probability_: upgrade_probability,
            max_height_: max_height,
        }
    }
}

impl<K> HeightControl<K> for GeometricalGenerator {
    fn max_height(&self) -> usize {
        self.max_height_
    }

    #[allow(unused_variables)]
    fn get_height(&mut self, key: &K) -> usize {
        // Simulates a random variate with geometric distribution. The idea is
        // that we are modelling number of successes until the first failure.
        let mut h = 0;

        while h < self.max_height_ {
            let rand::Open01(throw) = rand::random::<rand::Open01<f64>>();
            if throw >= self.upgrade_probability_ {
                return h;
            }

            h += 1;
        }

        h
    }
}

/// `HashCoinGenerator` creates heights by using a hash function that
/// distributes uniformly among the output universe and counting the number of
/// trailing zeros in the hashed value of a key. This is akin to using a
/// Geometric(1/2) when assuming the insertions are uniformly random; however,
/// it is faster than generating a random number for an unknown amount of time,
/// assuming that the hash function is not too expensive.
pub struct HashCoinGenerator<K, H> {
    max_height_: usize,
    hasher_: H,
    phantom_: std::marker::PhantomData<K>,
}

impl<K: std::hash::Hash, H: std::hash::Hasher> HashCoinGenerator<K, H> {
    /// Builds a new `HashCoinGenerator`
    ///
    /// # Arguments
    ///
    ///  * `max_height`: maximum height that the generator may give out to any
    ///    node.
    ///  * `hasher`: the hash function that will be used to generate the level
    ///    for a node. This should be from at least a 2-universal family.
    ///
    /// # Remarks
    ///
    /// The implementation can not and does not check for 2-universality on the
    /// hash function. A bad hash function may skew the generated heights
    /// towards bad distribution values and, in doing so, unbalance the skip
    /// list and affect its guarantees.
    ///
    /// As an example, if using this generator to build `u32` keys, and the
    /// hash function is the identity function, the `get_height` function will
    /// entirely depend on the input distribution.
    pub fn new(max_height: usize, hasher: H) -> HashCoinGenerator<K, H> {
        HashCoinGenerator {
            max_height_: max_height,
            hasher_: hasher,
            phantom_: std::marker::PhantomData,
        }
    }
}

impl<K: std::hash::Hash, H: std::hash::Hasher> HeightControl<K> for HashCoinGenerator<K, H> {
    fn max_height(&self) -> usize {
        self.max_height_
    }

    fn get_height(&mut self, key: &K) -> usize {
        // We expect the hash function to be uniformly distributed over the
        // output universe. This means that the probability of getting a
        // sequence of trailing zeros of zero-based length i is (1/2)^(i + 1)
        key.hash(&mut self.hasher_);
        // TODO: std::intrinsics::ctlz
        let height = self.hasher_.finish().trailing_zeros() as usize;
        // TODO: this is biased to low end values, unless max_height_ is a power
        // of two.
        height % self.max_height_
    }
}

/// `TwoPowGenerator` generates heights by simulating a capped geometrical
/// random variable, similar to `GeometricalGenerator`. This generator is
/// restricted to maximum heights that are powers of two and upgrades with
/// probability 1/2.
///
/// It should be preferred to `GeometricalGenerator` because the simulation is
/// done using only a single random throw.
pub struct TwoPowGenerator<K> {
    max_pow_: usize,
    phantom_: std::marker::PhantomData<K>,
}

impl<K> TwoPowGenerator<K> {
    pub fn new(max_height: usize) -> TwoPowGenerator<K> {
        assert!(max_height.is_power_of_two());

        TwoPowGenerator {
            max_pow_: max_height - 1,
            phantom_: std::marker::PhantomData,
        }
    }
}

impl<K> HeightControl<K> for TwoPowGenerator<K> {
    fn max_height(&self) -> usize {
        self.max_pow_ + 1
    }

    #[allow(unused_variables)]
    fn get_height(&mut self, key: &K) -> usize {
        // TODO: std::intrinsics::ctlz
        // The probability that a random value has a binary representation that
        // ends with 1 0^k is (1/2)^{k+1}.
        let height = rand::random::<usize>().trailing_zeros() as usize;
        // Since we are always doing `% 2^k` here, we are using the simple trick
        // exposed here: https://stackoverflow.com/q/6670715 .
        height & self.max_pow_
    }
}

impl<K: 'static + std::hash::Hash + Default> Default for SkipList<K> {
    fn default() -> Self {
        Self::new(Box::new(TwoPowGenerator::new(16)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: tests
}
