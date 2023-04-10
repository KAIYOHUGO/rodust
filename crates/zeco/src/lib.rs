pub mod des;
pub mod with;

pub use des::{Deserialize, Endian::*, SliceArg::*};
pub use with::{DeserializeWith, PrefixLen, TryTo};
pub use zeco_derive::Deserialize;
