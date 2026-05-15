#[cfg(feature = "smallvec")]
pub type SmallVec<T> = smallvec::SmallVec<[T; 1]>;
#[cfg(not(feature = "smallvec"))]
pub type SmallVec<T> = Vec<T>;
// #[cfg(feature = "smallvec")]
// pub use smallvec::smallvec;
// #[cfg(not(feature = "smallvec"))]
// pub use std::vec as smallvec;
