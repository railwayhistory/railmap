//! Paths, traces, and positions.

pub use self::path::{
    Distance, Location, MapDistance, Path, PathBuilder, SegTime
};
pub use self::trace::{
    Edge, Position, PartitionIter, SegmentIter, Subpath,Trace
};
pub(crate) use self::trace::CANVAS_ACCURACY;

mod path;
mod trace;

