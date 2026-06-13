use rfd::AsyncFileDialog;

pub struct FileSaver {
    dialog: AsyncFileDialog,
    name: String,
}

impl FileSaver {
    pub fn new() -> Self {
        Self {
            dialog: AsyncFileDialog::new(),
            name: String::from("download"),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.dialog = self.dialog.set_title(title);
        self
    }

    pub fn file_name(mut self, name: &str) -> Self {
        self.dialog = self.dialog.set_file_name(name);
        self.name = name.to_string();
        self
    }

    pub fn dispatch(self, data: Vec<u8>) {
        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::spawn(async move {
            if let Some(handle) = self.dialog.save_file().await {
                let _ = handle.write(&data).await;
            }
        });

        #[cfg(target_arch = "wasm32")]
        trigger_download(&self.name, &data);
    }
}

#[cfg(target_arch = "wasm32")]
fn trigger_download(filename: &str, data: &[u8]) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, HtmlAnchorElement, Url};

    let uint8 = js_sys::Uint8Array::from(data);
    let array = js_sys::Array::new();
    array.push(&uint8.buffer());

    let Ok(blob) = Blob::new_with_buffer_source_sequence(&array) else {
        return;
    };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else {
        return;
    };

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Ok(element) = document.create_element("a") else {
        return;
    };
    let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() else {
        return;
    };

    anchor.set_href(&url);
    anchor.set_download(filename);
    let _ = body.append_child(&anchor);
    anchor.click();
    let _ = body.remove_child(&anchor);
    let _ = Url::revoke_object_url(&url);
}
