use std::ops::BitOr;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Layer(pub u32);

impl Layer {
    pub const LAYER_0: Self = Self(1 << 0);
    pub const LAYER_1: Self = Self(1 << 1);
    pub const LAYER_2: Self = Self(1 << 2);
    pub const LAYER_3: Self = Self(1 << 3);
    pub const LAYER_4: Self = Self(1 << 4);
    pub const LAYER_5: Self = Self(1 << 5);
    pub const LAYER_6: Self = Self(1 << 6);
    pub const LAYER_7: Self = Self(1 << 7);
    pub const LAYER_8: Self = Self(1 << 8);
    pub const LAYER_9: Self = Self(1 << 9);
    pub const LAYER_10: Self = Self(1 << 10);
    pub const LAYER_11: Self = Self(1 << 11);
    pub const LAYER_12: Self = Self(1 << 12);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl BitOr for Layer {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
