use crate::float::Float;

#[inline]
pub fn single<T: Float>(a: T, b: &mut T) {
    if a < *b {
        *b = a;
    }
}

#[inline]
pub fn complete<T: Float>(a: T, b: &mut T) {
    if a > *b {
        *b = a;
    }
}

#[inline]
pub fn average<T: Float>(a: T, b: &mut T, size_a: usize, size_b: usize) {
    let size_a = T::from_usize(size_a);
    let size_b = T::from_usize(size_b);
    *b = (size_a * a + size_b * *b) / (size_a + size_b);
}

#[inline]
pub fn weighted<T: Float>(a: T, b: &mut T) {
    *b = T::from_float(0.5) * (a + *b);
}

#[inline]
pub fn ward<T: Float>(
    a: T,
    b: &mut T,
    merged_dist: T,
    size_a: usize,
    size_b: usize,
    size_x: usize,
) {
    let size_a = T::from_usize(size_a);
    let size_b = T::from_usize(size_b);
    let size_x = T::from_usize(size_x);

    let numerator = ((size_x + size_a) * a) + ((size_x + size_b) * *b)
        - (size_x * merged_dist);
    let denom = size_a + size_b + size_x;
    *b = numerator / denom;
}

#[inline]
pub fn centroid<T: Float>(
    a: T,
    b: &mut T,
    merged_dist: T,
    size_a: usize,
    size_b: usize,
) {
    let size_a = T::from_usize(size_a);
    let size_b = T::from_usize(size_b);
    let size_ab = size_a + size_b;

    *b = (((size_a * a) + (size_b * *b)) / size_ab)
        - ((size_a * size_b * merged_dist) / (size_ab * size_ab));
}

#[inline]
pub fn median<T: Float>(a: T, b: &mut T, merged_dist: T) {
    let half = T::from_float(0.5);
    let quarter = T::from_float(0.25);

    *b = (half * (a + *b)) - (merged_dist * quarter);
}
