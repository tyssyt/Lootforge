use crate::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode, IdbObjectStore};
use web_sys::js_sys::{Array, Uint8Array};
use crate::storage::storage_manager::{LoadingState, Store};

#[derive(Clone)]
pub struct WebStore {
    db: MaybeLoaded<IdbDatabase,IdbOpenDbRequest>,

    file_names: MaybeLoaded<Vec<String>, IdbRequest>,
    saving: Option<(String, IdbRequest)>,
    loading: Vec<(String, IdbRequest)>,
}
#[derive(Clone)]
enum MaybeLoaded<R,L> {
    Ready(R),
    Loading(L),
}
impl <R,L> MaybeLoaded<R,L> {
    fn is_loaded(&self) -> bool {        
        match self {
            MaybeLoaded::Ready(_) => true,
            MaybeLoaded::Loading(_) => false,
        }
    }
}
impl <R,L> Deref for MaybeLoaded<R,L> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        match self {
            MaybeLoaded::Ready(data) => data,
            MaybeLoaded::Loading(_) => panic!("Data accessed before loading is finished"),
        }
    }
}

const STORE: &'static str = "saves";

impl WebStore {
    pub fn new() -> WebStore {
        Self {
            db: MaybeLoaded::Loading(open_idb().expect("Could not access IndexedDB")),
            file_names: MaybeLoaded::Ready(Vec::new()), // a bit of a hack to avoid an Option
            saving: None,
            loading: Vec::new(),
        }
    }

    fn store(&self) -> IdbObjectStore {
        self.store_opt().expect("Could not open IDB Store")
    }
    fn store_opt(&self) -> Result<IdbObjectStore, JsValue> {
        let transaction = self.db.transaction_with_str_and_mode(STORE, IdbTransactionMode::Readwrite)?;
        transaction.object_store(STORE)
    }
}
impl Store for WebStore {    
    fn initializing(&mut self) -> bool {
        if self.db.is_loaded() && self.file_names.is_loaded() {
            return false;
        }

        match &self.db {
            MaybeLoaded::Ready(_) => {},
            MaybeLoaded::Loading(request) => {                
                if let Ok(db) = request.result() {
                    self.db = MaybeLoaded::Ready(db.dyn_into::<IdbDatabase>().unwrap());
                    self.file_names = MaybeLoaded::Loading(self.store().get_all_keys().unwrap());
                    return true;
                }
                if request.error().is_ok_and(|opt| opt.is_some()) {
                    panic!()
                }
                return true;
            },
        }

        match &self.file_names {            
            MaybeLoaded::Ready(_) => panic!(),
            MaybeLoaded::Loading(request) => {                                 
                if let Ok(keys) = request.result() {
                    let file_names = keys.dyn_into::<Array>().unwrap().iter()
                        .map(|key| key.as_string().unwrap())
                        .collect();
                    self.file_names = MaybeLoaded::Ready(file_names);
                    return false;
                }
                if request.error().is_ok_and(|opt| opt.is_some()) {
                    panic!()
                }
                return true;
            }
        }
    }

    fn file_names(&self) -> Vec<String> {
        self.file_names.deref().clone()
    }

    fn start_save(&mut self, file_name: String, bytes: Vec<u8>) {
        if let Some((other_file_name, _)) = &self.saving {
            warn!("Attempting to save {} while {} is still saving", file_name, other_file_name);
            return;
        }

        let key = JsValue::from_str(&file_name);
        let bytes = JsValue::from(Uint8Array::from(bytes.as_slice())); // TODO please someone show me reasonable docs, I wanna know what this does and why
        // TODO the fact that this is by ref makes me think he copies it, so maybe we want to use Uint8Array::view instead to avoid multiple copies
        let request = self.store().put_with_key(&bytes, &key).unwrap();
        self.saving = Some((file_name, request));        
        // TODO update file_names?
    }

    fn saving(&mut self) -> Option<String> {
        if let Some((key, request)) = &self.saving {                                          
            if request.result().is_ok() {
                self.saving = None;
                return None;
            }
            if let Ok(Some(error)) = request.error() {
                warn!("Error while saving {}: {:?}", key, error);
                self.saving = None;
                return None;
            }
            Some(key.clone()) // TODO could borrow I guess?
        } else {
            None
        }
    }

    fn start_load(&mut self, file_names: Vec<String>) {
        if self.loading.len() > 0 {
            panic!("Attempting to load {:?} while other file is loading", file_names);
        }

        let store = self.store();

        self.loading = file_names.into_iter()
            .map(|file| {
                let request = store.get(&JsValue::from_str(&file)).unwrap();
                (file, request)
            }).collect();
    }

    fn loading(&mut self) -> LoadingState<Vec<String>, Vec<Vec<u8>>> {
        if self.loading.is_empty() {
            return LoadingState::None;
        }

        if let Some((file, request)) = self.loading.iter().find(|(_, request) | request.error().is_ok_and(|opt| opt.is_some())) {
            panic!()
        }

        if !self.loading.iter().all(|(_, request)| request.result().is_ok()) {
            let file_names = self.loading.iter().map(|(file, _)| file.clone()).collect();
            return LoadingState::Loading(file_names);
        }

        let bytes = self.loading.drain(..)
            .map(|(_, request)| request.result().unwrap().dyn_into::<Uint8Array>().unwrap().to_vec())
            .collect();
        LoadingState::Done(bytes)
    }
}

fn open_idb() -> Option<IdbOpenDbRequest> {
    let idb_factory = web_sys::window()?.indexed_db().ok()??;
    let request = idb_factory.open("lootforge").ok()?;

    let upgrade = Closure::wrap(Box::new(move |event: web_sys::Event| {
        // TODO here I use .unwrap().unchecked_into(), above I use .dyn_into::<IdbDatabase>().unwrap()
        let request: IdbOpenDbRequest = event.target().unwrap().unchecked_into();
        let db: IdbDatabase = request.result().unwrap().unchecked_into();
        let _ = db.create_object_store(STORE);
    }) as Box<dyn FnMut(_)>);

    request.set_onupgradeneeded(Some(upgrade.into_js_value().unchecked_ref()));

    Some(request)
}
