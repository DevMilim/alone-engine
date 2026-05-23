#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TextureHandler(usize);
impl TextureHandler {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn id(self) -> usize {
        self.0
    }
}
