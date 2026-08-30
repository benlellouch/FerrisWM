mod atoms;
mod config;
mod effect;
mod ewmh_manager;
mod key_mapping;
mod keyboard;
mod layout;
mod state;
mod window_manager;
mod workspace;
mod x11;

fn main() {
    // SAFETY: called once at startup before any threads are spawned and before any
    // signal handlers are installed. Setting SIGCHLD to SIG_IGN instructs the kernel
    // to reap child processes automatically, preventing spawned clients from
    // accumulating as zombies in the WM's process table.
    unsafe {
        libc::signal(libc::SIGCHLD, libc::SIG_IGN);
    }

    env_logger::init();

    match window_manager::WindowManager::new() {
        Ok(mut wm) => {
            if let Err(e) = wm.run() {
                log::error!("Window manager runtime error: {e:?}");
            }
        }
        Err(e) => {
            log::error!("Failed to initialize window manager: {e:?}");
        }
    }
}
