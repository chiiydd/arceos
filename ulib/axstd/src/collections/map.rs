use hashbrown::hash_map as base;
use spinlock::SpinNoIrq;
#[allow(deprecated)]
use core::hash::{BuildHasher, Hash, Hasher, SipHasher13};

pub struct HashMap<K, V, S = RandomState> {
    base: base::HashMap<K, V, S>,

}

impl<K, V> HashMap<K, V, RandomState> {

    #[inline]
    #[must_use]
    pub fn new() -> HashMap<K, V, RandomState> {
        Default::default()
    }

}
impl <K,V,S> HashMap<K,V,S>{
    #[inline]
    pub const fn with_hasher(hash_builder: S) -> HashMap<K, V, S> {
        HashMap { base: base::HashMap::with_hasher(hash_builder) }
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    S: Default,
{
    /// Creates an empty `HashMap<K, V, S>`, with the `Default` value for the hasher.
    #[inline]
    fn default() -> HashMap<K, V, S> {
        HashMap::with_hasher(Default::default())
    }

    
}


pub struct RandomState {
    k0: u64,
    k1: u64,
}

impl RandomState {
    #[inline]
    #[allow(deprecated)]
    // rand
    #[must_use]
    pub fn new() -> RandomState {



        //TODO: get random numbers: k0 and k1.
        let ret =random();
        // test k0&k1
        // let k0=(ret>>64) as u64;
        // let k1=(ret&0x0000_0000_ffff_ffff_ffff_ffff )as u64;
        // println!("{}:{}",k0,k1);
        RandomState {
            k0:(ret>>64) as u64,
            k1:(ret&0x0000_0000_ffff_ffff_ffff_ffff )as u64,
        }
    }
}

impl BuildHasher for RandomState {
    type Hasher = DefaultHasher;
    #[inline]
    #[allow(deprecated)]
    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher(SipHasher13::new_with_keys(self.k0, self.k1))
    }
}

#[allow(deprecated)]
#[derive(Clone, Debug)]
pub struct DefaultHasher(SipHasher13);

impl DefaultHasher {
    /// Creates a new `DefaultHasher`.
    #[inline]
    #[allow(deprecated)]
    #[must_use]
    pub const fn new() -> DefaultHasher {
        DefaultHasher(SipHasher13::new_with_keys(0, 0))
    }
}

impl Default for DefaultHasher {
    /// Creates a new `DefaultHasher` using [`new`].
    /// See its documentation for more.
    #[inline]
    fn default() -> DefaultHasher {
        DefaultHasher::new()
    }
}

impl Hasher for DefaultHasher {
    // The underlying `SipHasher13` doesn't override the other
    // `write_*` methods, so it's ok not to forward them here.

    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.write(msg)
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        self.0.write_str(s);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

impl Default for RandomState {
    /// Constructs a new `RandomState`.
    #[inline]
    fn default() -> RandomState {
        RandomState::new()
    }
}


impl <K,V,S> HashMap<K,V,S> {
    #[rustc_lint_query_instability]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { base: self.base.iter() }
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    base: base::Iter<'a, K, V>,
}


impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
}


impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.base.insert(k, v)
    }
}


static PARK_MILLER_LEHMER_SEED:SpinNoIrq<u32>=SpinNoIrq::new(0);
// 2^32 -1=4294967295
const RAND_MAX: u64 = 4_294_967_295;
pub fn random()->u128{
    let mut seed =PARK_MILLER_LEHMER_SEED.lock();
    if *seed==0{
        *seed=arceos_api::time::ax_current_time().as_micros() as u32 ;
    }
    let mut ret:u128=0;

    for _ in 0..4{
        *seed = ((u64::from(*seed) * 48271) % RAND_MAX) as u32;
        ret= (ret << 32) | (*seed as u128);
    }
    ret 

}
