use crate::{FromInner, IntoInner};
use uv::{uv_prepare_init, uv_prepare_start, uv_prepare_stop, uv_prepare_t};

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct PrepareDataFields {
    prepare_cb: Option<Box<dyn FnMut(PrepareHandle)>>,
}

/// Callback for uv_prepare_start
extern "C" fn uv_prepare_cb(handle: *mut uv_prepare_t) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let super::PrepareData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.prepare_cb.as_mut() {
                    f(handle.into_inner());
                }
            }
        }
    }
}

/// Prepare handles will run the given callback once per loop iteration, right before polling for
/// i/o.
pub struct PrepareHandle {
    handle: *mut uv_prepare_t,
}

impl PrepareHandle {
    /// Create and initialize a new prepare handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<PrepareHandle> {
        let layout = std::alloc::Layout::new::<uv_prepare_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_prepare_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_prepare_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), super::PrepareData(Default::default()));

        Ok(PrepareHandle { handle })
    }

    /// Start the handle with the given callback.
    pub fn start(&mut self, cb: Option<impl FnMut(PrepareHandle) + 'static>) -> crate::Result<()> {
        // uv_cb is either Some(uv_prepare_cb) or None
        let uv_cb = cb.as_ref().map(|_| uv_prepare_cb as _);

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let super::PrepareData(d) = unsafe { &mut (*dataptr).addl } {
                d.prepare_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_prepare_start(self.handle, uv_cb) })
    }

    /// Stop the handle, the callback will no longer be called.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_prepare_stop(self.handle) })
    }
}

impl FromInner<*mut uv_prepare_t> for PrepareHandle {
    fn from_inner(handle: *mut uv_prepare_t) -> PrepareHandle {
        PrepareHandle { handle }
    }
}

impl IntoInner<*mut uv::uv_handle_t> for PrepareHandle {
    fn into_inner(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<PrepareHandle> for crate::Handle {
    fn from(prepare: PrepareHandle) -> crate::Handle {
        crate::Handle::from_inner(prepare.into_inner())
    }
}

impl crate::HandleTrait for PrepareHandle {}

impl crate::Loop {
    /// Create and initialize a new prepare handle
    pub fn prepare(&self) -> crate::Result<PrepareHandle> {
        PrepareHandle::new(self)
    }
}