fn main() {
    #[cfg(windows)]
    if let Err(error) = applifast_playback_probe::windows::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Apple playback probe is supported on Windows only.");
        std::process::exit(1);
    }
}
