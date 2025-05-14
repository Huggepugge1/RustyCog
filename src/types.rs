pub type CogId = i32;

pub trait CogType: Send + Clone + 'static {}
impl<T: Clone + Send + 'static> CogType for T {}
