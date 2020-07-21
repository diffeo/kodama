use num_traits::Float;

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
    let size_a = T::from(size_a).unwrap();
    let size_b = T::from(size_b).unwrap();
    *b = (size_a * a + size_b * *b) / (size_a + size_b);
}

#[inline]
pub fn weighted<T: Float>(a: T, b: &mut T) {
    *b = T::from(0.5).unwrap() * (a + *b);
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
    let size_a = T::from(size_a).unwrap();
    let size_b = T::from(size_b).unwrap();
    let size_x = T::from(size_x).unwrap();

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
    let size_a = T::from(size_a).unwrap();
    let size_b = T::from(size_b).unwrap();
    let size_ab = size_a + size_b;

    *b = (((size_a * a) + (size_b * *b)) / size_ab)
        - ((size_a * size_b * merged_dist) / (size_ab * size_ab));
}

#[inline]
pub fn median<T: Float>(a: T, b: &mut T, merged_dist: T) {
    let half = T::from(0.5).unwrap();
    let quarter = T::from(0.25).unwrap();

    *b = (half * (a + *b)) - (merged_dist * quarter);
}
