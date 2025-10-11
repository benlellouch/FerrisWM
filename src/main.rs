mod rdwm;

fn main() {
    let mut wm = rdwm::WindowManager::new();
    if let Err(e) = wm.run() {
        eprintln!("Window manager error: {:?}", e);
    }
}
