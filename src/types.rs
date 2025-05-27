pub type CogId = usize;
pub type EngineId = usize;

pub trait CogType: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> CogType for T {}
