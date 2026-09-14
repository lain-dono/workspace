pub mod id_internals;
pub mod iter;
pub mod kernel;

pub use self::iter::*;
pub use self::kernel::{
    Connectivity, EdgeId, EdgeIdRange, FaceId, FaceIdRange, VertId, VertIdRange,
};
pub use sid::{Id, IdRange, IdVec};
