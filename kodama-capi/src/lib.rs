use std::slice;

use kodama::{linkage, Method};

#[macro_use]
mod macros;

#[allow(non_camel_case_types)]
pub type size_t = usize;
#[allow(non_camel_case_types)]
pub type c_float = f32;
#[allow(non_camel_case_types)]
pub type c_double = f64;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum kodama_method {
    Single,
    Complete,
    Average,
    Weighted,
    Ward,
    Centroid,
    Median,
}

#[repr(C)]
#[derive(Debug)]
pub struct kodama_dendrogram {
    steps: Vec<kodama_step>,
    observations: size_t,
}

#[repr(C)]
#[derive(Debug)]
pub struct kodama_step {
    pub cluster1: size_t,
    pub cluster2: size_t,
    pub dissimilarity: c_double,
    pub size: size_t,
}

impl kodama_method {
    fn into_method(self) -> Method {
        match self {
            kodama_method::Single => Method::Single,
            kodama_method::Complete => Method::Complete,
            kodama_method::Average => Method::Average,
            kodama_method::Weighted => Method::Weighted,
            kodama_method::Ward => Method::Ward,
            kodama_method::Centroid => Method::Centroid,
            kodama_method::Median => Method::Median,
        }
    }
}

ffi_fn! {
    fn kodama_linkage_double(
        dis: *mut c_double,
        observations: size_t,
        method: kodama_method,
    ) -> *mut kodama_dendrogram {
        assert!(!dis.is_null());
        let dis_len = (observations * (observations - 1)) / 2;
        let dis = unsafe { slice::from_raw_parts_mut(dis, dis_len) };
        let dend = linkage(dis, observations, method.into_method());

        let mut c_steps = vec![];
        for step in dend.steps() {
            c_steps.push(kodama_step {
                cluster1: step.cluster1,
                cluster2: step.cluster2,
                dissimilarity: step.dissimilarity,
                size: step.size,
            });
        }
        Box::into_raw(Box::new(kodama_dendrogram {
            steps: c_steps,
            observations: observations,
        }))
    }
}

ffi_fn! {
    fn kodama_linkage_float(
        dis: *mut c_float,
        observations: size_t,
        method: kodama_method,
    ) -> *mut kodama_dendrogram {
        assert!(!dis.is_null());
        let dis_len = (observations * (observations - 1)) / 2;
        let dis = unsafe { slice::from_raw_parts_mut(dis, dis_len) };
        let dend = linkage(dis, observations, method.into_method());

        let mut c_steps = vec![];
        for step in dend.steps() {
            c_steps.push(kodama_step {
                cluster1: step.cluster1,
                cluster2: step.cluster2,
                dissimilarity: step.dissimilarity as c_double,
                size: step.size,
            });
        }
        Box::into_raw(Box::new(kodama_dendrogram {
            steps: c_steps,
            observations: observations,
        }))
    }
}

ffi_fn! {
    fn kodama_dendrogram_free(
        dend: *mut kodama_dendrogram,
    ) {
        unsafe { Box::from_raw(dend); }
    }
}

ffi_fn! {
    fn kodama_dendrogram_len(
        dend: *const kodama_dendrogram,
    ) -> size_t {
        let dend = unsafe { &*dend };
        dend.steps.len()
    }
}

ffi_fn! {
    fn kodama_dendrogram_observations(
        dend: *const kodama_dendrogram,
    ) -> size_t {
        let dend = unsafe { &*dend };
        dend.observations
    }
}

ffi_fn! {
    fn kodama_dendrogram_steps(
        dend: *const kodama_dendrogram,
    ) -> *const kodama_step {
        let dend = unsafe { &*dend };
        dend.steps.as_ptr()
    }
}
