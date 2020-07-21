#![feature(test)]

extern crate byteorder;
extern crate kodama;
#[macro_use]
extern crate lazy_static;
extern crate test;

use byteorder::{ByteOrder, LittleEndian};
use kodama::{Method, MethodChain};
use test::Bencher;

lazy_static! {
    static ref MA_CONDENSED_DISTS_SMALL: (Vec<f64>, usize) = {
        const DIST_BYTES: &'static [u8] =
            include_bytes!("../data/locations/ma-bench-small.dist");

        let mut dists = vec![0.0; DIST_BYTES.len() / 8];
        unsafe {
            LittleEndian::read_f64_into_unchecked(DIST_BYTES, &mut dists);
        }
        (dists, 200)
    };
    static ref MA_CONDENSED_DISTS_LARGE: (Vec<f64>, usize) = {
        const DIST_BYTES: &'static [u8] =
            include_bytes!("../data/locations/ma-bench-large.dist");

        let mut dists = vec![0.0; DIST_BYTES.len() / 8];
        unsafe {
            LittleEndian::read_f64_into_unchecked(DIST_BYTES, &mut dists);
        }
        (dists, 2000)
    };
}

macro_rules! bench_linkage {
    ($name:ident, $dists:expr, mst) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let dists = &$dists;
            b.iter(|| {
                let mut dis = dists.0.clone();
                let dend = kodama::mst(&mut dis, dists.1);
                assert_eq!(dend.len(), dists.1 - 1);
            });
        }
    };
    ($name:ident, $dists:expr, $method:expr, $other:ident) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let dists = &$dists;
            b.iter(|| {
                let mut dis = dists.0.clone();
                let dend = kodama::$other(&mut dis, dists.1, $method);
                assert_eq!(dend.len(), dists.1 - 1);
            });
        }
    };
}

// mst on small and large data
bench_linkage!(single_ma_small_mst, MA_CONDENSED_DISTS_SMALL, mst);
bench_linkage!(single_ma_large_mst, MA_CONDENSED_DISTS_LARGE, mst);

// nnchain on small data
bench_linkage!(
    single_ma_small_nnchain,
    MA_CONDENSED_DISTS_SMALL,
    MethodChain::Single,
    nnchain
);
bench_linkage!(
    complete_ma_small_nnchain,
    MA_CONDENSED_DISTS_SMALL,
    MethodChain::Complete,
    nnchain
);
bench_linkage!(
    average_ma_small_nnchain,
    MA_CONDENSED_DISTS_SMALL,
    MethodChain::Average,
    nnchain
);
bench_linkage!(
    weighted_ma_small_nnchain,
    MA_CONDENSED_DISTS_SMALL,
    MethodChain::Weighted,
    nnchain
);
bench_linkage!(
    ward_ma_small_nnchain,
    MA_CONDENSED_DISTS_SMALL,
    MethodChain::Ward,
    nnchain
);

// nnchain on large data
bench_linkage!(
    single_ma_large_nnchain,
    MA_CONDENSED_DISTS_LARGE,
    MethodChain::Single,
    nnchain
);
bench_linkage!(
    complete_ma_large_nnchain,
    MA_CONDENSED_DISTS_LARGE,
    MethodChain::Complete,
    nnchain
);
bench_linkage!(
    average_ma_large_nnchain,
    MA_CONDENSED_DISTS_LARGE,
    MethodChain::Average,
    nnchain
);
bench_linkage!(
    weighted_ma_large_nnchain,
    MA_CONDENSED_DISTS_LARGE,
    MethodChain::Weighted,
    nnchain
);
bench_linkage!(
    ward_ma_large_nnchain,
    MA_CONDENSED_DISTS_LARGE,
    MethodChain::Ward,
    nnchain
);

// generic on small data
bench_linkage!(
    single_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Single,
    generic
);
bench_linkage!(
    complete_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Complete,
    generic
);
bench_linkage!(
    average_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Average,
    generic
);
bench_linkage!(
    weighted_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Weighted,
    generic
);
bench_linkage!(
    ward_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Ward,
    generic
);
bench_linkage!(
    centroid_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Centroid,
    generic
);
bench_linkage!(
    median_ma_small_generic,
    MA_CONDENSED_DISTS_SMALL,
    Method::Median,
    generic
);

// generic on large data
bench_linkage!(
    single_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Single,
    generic
);
bench_linkage!(
    complete_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Complete,
    generic
);
bench_linkage!(
    average_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Average,
    generic
);
bench_linkage!(
    weighted_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Weighted,
    generic
);
bench_linkage!(
    ward_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Ward,
    generic
);
bench_linkage!(
    centroid_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Centroid,
    generic
);
bench_linkage!(
    median_ma_large_generic,
    MA_CONDENSED_DISTS_LARGE,
    Method::Median,
    generic
);

// primitive on small data
bench_linkage!(
    single_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Single,
    primitive
);
bench_linkage!(
    complete_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Complete,
    primitive
);
bench_linkage!(
    average_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Average,
    primitive
);
bench_linkage!(
    weighted_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Weighted,
    primitive
);
bench_linkage!(
    ward_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Ward,
    primitive
);
bench_linkage!(
    centroid_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Centroid,
    primitive
);
bench_linkage!(
    median_ma_small_primitive,
    MA_CONDENSED_DISTS_SMALL,
    Method::Median,
    primitive
);
