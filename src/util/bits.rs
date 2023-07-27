#[inline]
pub const fn is_aligned(value: usize, alignment_size: usize) -> bool {
    value % alignment_size == 0
}

#[allow(unused)]
#[inline]
pub const fn is_power_of_2(value: usize) -> bool {
    value > 0 && (value & (value - 1)) == 0
}

#[inline]
pub const fn min_aligned_size(floor: usize, alignment_size: usize) -> usize {
    if is_aligned(floor, alignment_size) {
        floor
    } else {
        ((floor / alignment_size) + 1) * alignment_size
    }
}

#[inline]
pub const fn max_aligned_size(ceil: usize, alignment_size: usize) -> usize {
    (ceil / alignment_size) * alignment_size
}
