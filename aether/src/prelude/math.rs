use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
pub struct Size<T : Debug + Default>
{
    pub width: T,
    pub height: T
}
