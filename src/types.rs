pub type CogId = i32;

pub trait CogType: Send + 'static {}
impl<T: Send + 'static> CogType for T {}
