use std::cell::OnceCell;

pub trait AppState: Sized + Default + 'static {
    unsafe fn state_cell() -> &'static mut OnceCell<Self>;

    fn read<'a>() -> &'a Self {
        unsafe { AppState::state_cell().get_or_init(|| Self::default()) }
    }

    fn update<F: Fn(&mut Self)>(f: F) {
        unsafe {
            let cell = AppState::state_cell();

            if cell.get().is_none() {
                let _ = cell.set(Self::default());
            }

            f(cell.get_mut().unwrap())
        }
    }
}
