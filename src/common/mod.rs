pub mod messages;
pub mod options;
pub mod parsing;

pub use self::options::SearchOptions;

use rand;
use rand::distributions::IndependentSample;

pub fn random_port() -> u16 {
    let port_range = rand::distributions::Range::new(32_768_u16, 65_535_u16);
    let mut rng = rand::thread_rng();
    port_range.ind_sample(&mut rng)
}
