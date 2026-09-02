use std::ops::BitOr;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Layer(pub u32);

impl Layer {
    pub const LAYER_0: Self = Self(0);
    pub const LAYER_1: Self = Self(1);
    pub const LAYER_2: Self = Self(2);
    pub const LAYER_3: Self = Self(3);
}

impl BitOr for Layer {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
