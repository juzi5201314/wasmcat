pub mod decode;
pub mod error;
pub mod instruction;
pub mod module;
pub mod parser;
pub mod section;
pub mod types;

type SVec<T> = smallvec::SmallVec<[T; 4]>;
