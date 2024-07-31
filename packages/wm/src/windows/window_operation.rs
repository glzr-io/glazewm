#[derive(Debug, Clone, Default)]
pub struct WindowOperation{
    pub operation: Operation,

}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum Operation{
    #[default]
    None,
    Waiting,
    Moving,
    Resizing,
}
