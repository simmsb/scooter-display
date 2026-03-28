// use core::cell::RefCell;

// use alloc::rc::Rc;

// struct DisplayBackend {
//     window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
//     buffer_provider: RefCell<DrawBuffer>,
//     touch: RefCell<Touch>,
//     backlight: RefCell<Option<Backlight>>,
// }

// impl slint::platform::Platform for DisplayBackend {
//     fn create_window_adapter(
//         &self,
//     ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
//         todo!()
//     }
// }
