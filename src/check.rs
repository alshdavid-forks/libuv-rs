use uv::{uv_check_init, uv_check_start, uv_check_stop, uv_check_t};

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct CheckDataFields {
    check_cb: Option<Box<dyn FnMut(CheckHandle)>>,
}

/// Callback for uv_check_start
extern "C" fn check_cb(handle: *mut uv_check_t) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let crate::CheckData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.check_cb.as_mut() {
                    f(handle.into());
                }
            }
        }
    }
}

/// Check handles will run the given callback once per loop iteration, right after polling for i/o.
pub struct CheckHandle {
    handle: *mut uv_check_t,
    should_drop: bool,
}

impl CheckHandle {
    /// Create and initialize a new check handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<CheckHandle> {
        let layout = std::alloc::Layout::new::<uv_check_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_check_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_check_init(r#loop.into(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), crate::CheckData(Default::default()));

        Ok(CheckHandle {
            handle,
            should_drop: true,
        })
    }

    /// Start the handle with the given callback.
    pub fn start(&mut self, cb: Option<impl FnMut(CheckHandle) + 'static>) -> crate::Result<()> {
        // uv_cb is either Some(check_cb) or None
        let uv_cb = cb.as_ref().map(|_| check_cb as _);

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let crate::CheckData(d) = unsafe { &mut (*dataptr).addl } {
                d.check_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_check_start(self.handle, uv_cb) })
    }

    /// Stop the handle, the callback will no longer be called.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_check_stop(self.handle) })
    }
}

impl From<*mut uv_check_t> for CheckHandle {
    fn from(handle: *mut uv_check_t) -> CheckHandle {
        CheckHandle {
            handle,
            should_drop: false,
        }
    }
}

impl From<CheckHandle> for crate::Handle {
    fn from(check: CheckHandle) -> crate::Handle {
        (check.handle as *mut uv::uv_handle_t).into()
    }
}

impl Into<*mut uv::uv_handle_t> for CheckHandle {
    fn into(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl crate::HandleTrait for CheckHandle {}

impl Drop for CheckHandle {
    fn drop(&mut self) {
        if self.should_drop {
            if !self.handle.is_null() {
                crate::Handle::free_data(uv_handle!(self.handle));

                let layout = std::alloc::Layout::new::<uv_check_t>();
                unsafe { std::alloc::dealloc(self.handle as _, layout) };
            }
            self.handle = std::ptr::null_mut();
        }
    }
}

impl crate::Loop {
    /// Create and initialize a new check handle
    pub fn check(&self) -> crate::Result<CheckHandle> {
        CheckHandle::new(self)
    }
}
