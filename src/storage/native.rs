use crate::prelude::*;
use std::sync::atomic::Ordering;
use std::{path::PathBuf, sync::atomic::AtomicBool};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use super::storage_manager::{LoadingState, Store};


pub struct NativeStore {
    thread: thread::JoinHandle<()>,
    channel: mpsc::Sender<Job>,

    saving: Option<Arc<SaveJob>>,
    loading: Option<Arc<LoadJob>>,
}

struct LoadJob {
    file_names: Vec<String>,
    bytes: Mutex<Vec<Vec<u8>>>,
    done: AtomicBool,
}
struct SaveJob {
    file_name: String,
    bytes: Vec<u8>,
    done: AtomicBool,
}
enum Job {
    Load(Arc<LoadJob>),
    Save(Arc<SaveJob>),
    Quit,
}
impl LoadJob {
    fn run(&self) -> std::io::Result<()> {
        let loaded = self.file_names.iter()
            .map(|file| std::fs::read(storage_dir().join(file)))
            .collect::<Result<Vec<_>,_>>()?;
        {
            let mut bytes = self.bytes.lock().unwrap();
            *bytes = loaded;
        }
        self.done.store(true, Ordering::Relaxed);
        Ok(())
    }
}
impl SaveJob {    
    fn run(&self) -> std::io::Result<()> {        
        let path = storage_dir();
        std::fs::create_dir_all(&path)?;
        std::fs::write(path.join(&self.file_name), &self.bytes)?;

        self.done.store(true, Ordering::Relaxed);
        Ok(())
    }
}

impl NativeStore {
    pub fn new() -> NativeStore {
        let (thread, channel) = Self::start_thread();
        NativeStore { thread, channel, saving: None, loading: None }
    }

    fn start_thread() -> (thread::JoinHandle<()>, mpsc::Sender<Job>) {        
        let (job_sender, job_receiver) = mpsc::channel::<Job>();
        let thread = thread::spawn(move || {
            loop {
                match job_receiver.recv().unwrap() {
                    Job::Load(job) => job.run().unwrap(),
                    Job::Save(job) => job.run().unwrap(),
                    Job::Quit => break,
                }
            }
        });
        (thread, job_sender)
    }

    fn check_thread(&mut self) {
        if self.thread.is_finished() {
            panic!("NativeStore worker thread died!"); // TODO recover
        }
    }
}

impl Store for NativeStore {
    fn initializing(&mut self) -> bool {
        false
    }

    fn file_names(&self) -> Vec<String> {
        if let Some(dir) = std::fs::read_dir(storage_dir()).ok() {
            dir
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|filename| filename.ends_with(".save"))
            .collect()
        } else {
            Vec::new()
        }
    }

    fn start_save(&mut self, file_name: String, bytes: Vec<u8>) {
        if self.saving.is_some() {
            warn!("Attempting to save {} while {} is still saving", file_name, self.saving.as_ref().unwrap().file_name);
            return;
        }

        self.check_thread();

        let job = Arc::new(SaveJob { file_name, bytes, done: AtomicBool::new(false) });
        self.saving = Some(job.clone());
        self.channel.send(Job::Save(job)).unwrap();
    }

    fn saving(&mut self) -> Option<String> {
        if let Some(job) = &self.saving {
            if job.done.load(Ordering::Relaxed) {
                self.saving = None;
                return None;
            }
            Some(job.file_name.clone())
        } else {
            None
        }
    }

    fn start_load(&mut self, file_names: Vec<String>) {
        if self.loading.is_some() {
            panic!("Attempting to load {:?} while {:?} is still loading", file_names, self.loading.as_ref().unwrap().file_names);
        }

        self.check_thread();

        let job = Arc::new(LoadJob { file_names, bytes: Mutex::new(Vec::new()), done: AtomicBool::new(false) });
        self.loading = Some(job.clone());
        self.channel.send(Job::Load(job)).unwrap();
    }

    fn loading(&mut self) -> LoadingState<Vec<String>, Vec<Vec<u8>>> {        
        if let Some(job) = &self.loading {
            if job.done.load(Ordering::Relaxed) {
                // TODO use replace once it is no longer nightly https://github.com/rust-lang/rust/issues/133407
                let bytes = {
                    let mut bytes = job.bytes.lock().unwrap();
                    std::mem::take(bytes.as_mut())
                };
                self.loading = None;
                return LoadingState::Done(bytes);
            }
            LoadingState::Loading(job.file_names.clone())
        } else {
            LoadingState::None
        }
    }
}

impl Drop for NativeStore {
    fn drop(&mut self) {
        self.channel.send(Job::Quit).unwrap();
    }
}

fn storage_dir() -> PathBuf {
    storage_dir_opt("lootforge").expect("could not access file system")
}

// from eframe

/// The folder where `eframe` will store its state.
///
/// The given `app_id` is either the
/// [`egui::ViewportBuilder::app_id`] of [`crate::NativeOptions::viewport`]
/// or the title argument to [`crate::run_native`].
///
/// On native, the path is:
/// * Linux:   `/home/UserName/.local/share/APP_ID`
/// * macOS:   `/Users/UserName/Library/Application Support/APP_ID`
/// * Windows: `C:\Users\UserName\AppData\Roaming\APP_ID\data`
fn storage_dir_opt(app_id: &str) -> Option<PathBuf> {
    use egui::os::OperatingSystem as OS;
    use std::env::var_os;
    match OS::from_target_os() {
        OS::Nix => var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| home::home_dir().map(|p| p.join(".local").join("share")))
            .map(|p| {
                p.join(
                    app_id
                        .to_lowercase()
                        .replace(|c: char| c.is_ascii_whitespace(), ""),
                )
            }),
        OS::Mac => home::home_dir().map(|p| {
            p.join("Library")
                .join("Application Support")
                .join(app_id.replace(|c: char| c.is_ascii_whitespace(), "-"))
        }),
        OS::Windows => roaming_appdata().map(|p| p.join(app_id).join("data")),
        OS::Unknown | OS::Android | OS::IOS => None,
    }
}


// Adapted from
// https://github.com/rust-lang/cargo/blob/6e11c77384989726bb4f412a0e23b59c27222c34/crates/home/src/windows.rs#L19-L37
#[cfg(all(windows, not(target_vendor = "uwp")))]
#[allow(unsafe_code)]
fn roaming_appdata() -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;
    use std::slice;

    use windows_sys::Win32::Foundation::S_OK;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_RoamingAppData, SHGetKnownFolderPath, KF_FLAG_DONT_VERIFY,
    };

    extern "C" {
        fn wcslen(buf: *const u16) -> usize;
    }
    let mut path_raw = ptr::null_mut();

    // SAFETY: SHGetKnownFolderPath allocates for us, we don't pass any pointers to it.
    // See https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath
    let result = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_RoamingAppData,
            KF_FLAG_DONT_VERIFY as u32,
            std::ptr::null_mut(),
            &mut path_raw,
        )
    };

    let path = if result == S_OK {
        // SAFETY: SHGetKnownFolderPath indicated success and is supposed to allocate a nullterminated string for us.
        let path_slice = unsafe { slice::from_raw_parts(path_raw, wcslen(path_raw)) };
        Some(PathBuf::from(OsString::from_wide(path_slice)))
    } else {
        None
    };

    // SAFETY:
    // This memory got allocated by SHGetKnownFolderPath, we didn't touch anything in the process.
    // A null ptr is a no-op for `CoTaskMemFree`, so in case this failed we're still good.
    // https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cotaskmemfree
    unsafe { CoTaskMemFree(path_raw.cast()) };

    path
}

#[cfg(any(not(windows), target_vendor = "uwp"))]
fn roaming_appdata() -> Option<PathBuf> {
    None
}