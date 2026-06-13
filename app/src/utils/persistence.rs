use lemon_noise::persistence::PersistenceBackend;

pub fn default_backend() -> Box<dyn PersistenceBackend> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(native::FileBackend::new())
    }
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(web::LocalStorageBackend)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use lemon_noise::persistence::PersistenceBackend;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct FileBackend {
        path: PathBuf,
    }

    impl FileBackend {
        pub fn new() -> Self {
            let path =
                directories::ProjectDirs::from("io.github", "zitronenjoghurt", "lemon-noise")
                    .map(|dirs| dirs.data_dir().join("state.bin"))
                    .unwrap_or_else(|| PathBuf::from("lemon-noise-state.bin"));
            Self { path }
        }
    }

    impl PersistenceBackend for FileBackend {
        fn load(&self) -> Option<Vec<u8>> {
            std::fs::read(&self.path).ok()
        }

        fn save(&self, data: &[u8]) {
            if let Some(parent) = self.path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let tmp = self.path.with_extension("tmp");
            if std::fs::write(&tmp, data).is_ok() {
                let _ = std::fs::rename(&tmp, &self.path);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use lemon_noise::persistence::PersistenceBackend;

    const STORAGE_KEY: &str = "lemon-noise-state";

    #[derive(Debug)]
    pub struct LocalStorageBackend;

    impl LocalStorageBackend {
        fn storage() -> Option<web_sys::Storage> {
            web_sys::window()?.local_storage().ok()?
        }
    }

    impl PersistenceBackend for LocalStorageBackend {
        fn load(&self) -> Option<Vec<u8>> {
            let raw = Self::storage()?.get_item(STORAGE_KEY).ok().flatten()?;
            BASE64.decode(raw).ok()
        }

        fn save(&self, data: &[u8]) {
            if let Some(storage) = Self::storage() {
                let _ = storage.set_item(STORAGE_KEY, &BASE64.encode(data));
            }
        }
    }
}
