use crate::{FromInner, IntoInner};
use uv::uv_timespec_t;

pub struct TimeSpec {
    sec: i64,
    nsec: i64,
}

impl FromInner<uv_timespec_t> for TimeSpec {
    fn from_inner(ts: uv_timespec_t) -> TimeSpec {
        TimeSpec {
            sec: ts.tv_sec,
            nsec: ts.tv_nsec,
        }
    }
}