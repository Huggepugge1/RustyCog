pub type CogId = usize;
pub type EngineId = usize;

pub trait CogType: Send + 'static {}
impl<T: Send + 'static> CogType for T {}
