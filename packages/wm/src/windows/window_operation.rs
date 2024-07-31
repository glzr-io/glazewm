#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum WindowOperation{
    #[default]
    None,
    Waiting,
    Moving,
    Resizing,
}
