pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub fn is_one_f32(t: &f32) -> bool {
    t == &1.0
}

pub fn is_zero_f32(t: &f32) -> bool {
    t == &0.0
}

pub fn is_one_f32x2(t: &[f32; 2]) -> bool {
    t == &[1.0; 2]
}

pub fn is_zero_f32x2(t: &[f32; 2]) -> bool {
    t == &[0.0; 2]
}
