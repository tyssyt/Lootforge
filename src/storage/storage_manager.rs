use crate::prelude::*;
use crate::storage::ser::ValidatedBytes;
use crate::{storage::ser_v1, LootforgeApp};

#[apply(Enum)]
#[derive(PartialEq)]
pub enum LoadingState<L,D> {
    None,
    Loading(L),
    Done(D),
}
pub trait Store {
    fn initializing(&mut self) -> bool;
    fn file_names(&self) -> Vec<String>;

    fn start_save(&mut self, file_name: String, bytes: Vec<u8>);
    fn saving(&mut self) -> Option<String>;

    fn start_load(&mut self, file_name: Vec<String>);
    fn loading(&mut self) -> LoadingState<Vec<String>, Vec<Vec<u8>>>;
}

#[derive(Debug)]
pub struct StorageManager {
    #[cfg(target_arch = "wasm32")]
    store: super::web::WebStore,
    #[cfg(not(target_arch = "wasm32"))]
    store: super::native::NativeStore,

    save0: bool,
    pub load_quicksaves_after_init: bool,
}
impl Default for StorageManager {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        return Self { store: super::web::WebStore::new(), save0: true, load_quicksaves_after_init: false };
        #[cfg(not(target_arch = "wasm32"))]
        return Self { store: super::native::NativeStore::new(), save0: true, load_quicksaves_after_init: false };
    }
}
impl Clone for StorageManager {
    fn clone(&self) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            store: self.store.clone(),
            #[cfg(not(target_arch = "wasm32"))]
            store: super::native::NativeStore::new(),
            save0: self.save0,
            load_quicksaves_after_init: self.load_quicksaves_after_init,
        }
    }
}

impl StorageManager {
    pub fn saving(&mut self) -> bool {
        self.store.saving().is_some()
    }

    pub fn maybe_save(app: &mut LootforgeApp) {
        if app.storage_manager.saving() || !app.timekeeper.needs_save() {
            return;
        }
        Self::def_save(app);
    }

    pub fn def_save(app: &mut LootforgeApp) {
        let epoch_millis = app.timekeeper.save_now();

        if app.storage_manager.save0 {
            StorageManager::save_app("auto0.save".to_owned(), app, epoch_millis);
            app.storage_manager.save0 = false;
        } else {
            StorageManager::save_app("auto1.save".to_owned(), app, epoch_millis);
            app.storage_manager.save0 = true;
        }
    }

    fn save_app(file_name: String, app: &mut LootforgeApp, epoch_millis: u64) {
        let bytes = ser_v1::ser(app, epoch_millis);
        app.storage_manager.save_bytes(file_name, bytes);
    }
    
    fn save_bytes(&mut self, file_name: String, bytes: Vec<u8>) {
        info!("saving to {}", file_name);
        // TODO compress but also that should be async...
        
        self.store.start_save(file_name, bytes);
    }

    pub fn loading(app: &mut LootforgeApp) -> bool {
        if app.storage_manager.store.initializing() {
            return true;
        }

        let loading = app.storage_manager.store.loading();

        if loading == LoadingState::None && app.storage_manager.load_quicksaves_after_init {
            app.storage_manager.start_loading_quicksaves();
            app.storage_manager.load_quicksaves_after_init = false;
            return true;
        }

        match loading {
            LoadingState::None => return false,
            LoadingState::Loading(_) => return true,
            LoadingState::Done(bytes) => {
                if let Some(loaded_app) = app.storage_manager.load_saves(bytes) {
                    app.storage_manager.manage_saves();
                    *app = loaded_app;
                    info!("load successful");
                } else {
                    warn!("Failed to load last save, starting new game");
                    app.stash.give_starting_items();
                }
                return false;
            },
        }
    }

    fn start_loading_quicksaves(&mut self) {
        let quicksaves: Vec<_> = self.store.file_names()
            .into_iter()
            .filter(|save| save.starts_with("auto"))
            .collect();

        if quicksaves.is_empty() {
            info!("no quicksave found, starting new game");
            return;
        }

        info!("loading {:?}", &quicksaves);
        self.store.start_load(quicksaves);
    }

    fn load_saves(&self, bytes: Vec<Vec<u8>>) -> Option<LootforgeApp> {
        // TODO decompress
        bytes.into_iter()
            .filter_map(|bytes| ValidatedBytes::validate(bytes))
            .max_by_key(|v| (v.version, v.ts))
            .and_then(|v| match v.version {
                1 => ser_v1::deser(self.clone(), &v.bytes),
                _ => panic!(),
            })
    }

    fn manage_saves(&mut self) {}
    // then it copies the loaded autosave to a timestamped file (only if it does not exist? because if shit fails we might load more then once)
    
    // then it deletes the oldest timestamped file if there are more then 10
    // but I guess I can keep 1 a month?
    
    // after the timeskip we will automatically do the first autosave
    
    // Problem scenario: shit is broke and user keeps reloading, overwriting the last 10 saves
    
    // we keep
    // the newest save
    // the newest save older 1h
    // the newest save older 3h
    // the newest save older 6h
    // the newest save older 12h
    // the newest save older 24h
    // the newest save older 48h
    // the newest save older 120h
    // the newest save older 240h
}

