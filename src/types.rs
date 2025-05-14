use std::fmt::Debug;

pub type CogId = i32;

pub trait CogType: Clone + Debug + Send + 'static {}
impl<T: Clone + Debug + Send + 'static> CogType for T {}
