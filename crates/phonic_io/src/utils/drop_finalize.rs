use crate::{delegate_format, FormatWriter};

pub struct DropFinalize<T: FormatWriter>(pub T);

delegate_format! {
    impl<T: FormatWriter> * for DropFinalize<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: FormatWriter> Drop for DropFinalize<T> {
    fn drop(&mut self) {
        let _ = self.finalize();
        let _ = self.flush();
    }
}
