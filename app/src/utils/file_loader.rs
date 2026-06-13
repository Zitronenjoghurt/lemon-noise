use rfd::AsyncFileDialog;
use std::sync::mpsc::Sender;

pub struct FileLoader {
    dialog: AsyncFileDialog,
}

impl FileLoader {
    pub fn new() -> Self {
        Self {
            dialog: AsyncFileDialog::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.dialog = self.dialog.set_title(title);
        self
    }

    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.dialog = self.dialog.add_filter(name, extensions);
        self
    }

    pub fn dispatch(self, tx: Sender<Vec<u8>>) {
        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::spawn(async move {
            if let Some(handle) = self.dialog.pick_file().await {
                let data = handle.read().await;
                let _ = tx.send(data);
            }
        });

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(handle) = self.dialog.pick_file().await {
                let data = handle.read().await;
                let _ = tx.send(data);
            }
        });
    }
}
