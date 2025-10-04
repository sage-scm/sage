use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub const PROGRESS_FRAMES: &[&str] = &["◡◡◡", "◠◡◡", "◡◠◡", "◡◡◠", "◡◠◡", "◠◡◡", "◡◡◡"];
const PROGRESS_FRAME_INTERVAL: Duration = Duration::from_millis(120);

pub struct ProgressIndicator {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    needs_clear: Arc<AtomicBool>,
    use_spinner: bool,
    finished: bool,
}

impl ProgressIndicator {
    pub(crate) fn spinner(
        message: String,
        frames: Vec<String>,
        needs_clear: Arc<AtomicBool>,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        needs_clear.store(true, Ordering::SeqCst);

        let stop_for_thread = stop.clone();
        let message_for_thread = Arc::new(message);
        let frames_for_thread = Arc::new(frames);

        let handle = thread::spawn(move || {
            let mut index = 0usize;
            while !stop_for_thread.load(Ordering::SeqCst) {
                let frame = &frames_for_thread[index % frames_for_thread.len()];
                index = index.wrapping_add(1);

                {
                    let mut stdout = io::stdout().lock();
                    let _ = write!(stdout, "\r  {} {}", frame, message_for_thread.as_str());
                    let _ = stdout.flush();
                }

                thread::sleep(PROGRESS_FRAME_INTERVAL);
            }
        });

        Self {
            stop,
            handle: Some(handle),
            needs_clear,
            use_spinner: true,
            finished: false,
        }
    }

    pub(crate) fn noop(needs_clear: Arc<AtomicBool>) -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(true)),
            handle: None,
            needs_clear,
            use_spinner: false,
            finished: true,
        }
    }

    pub fn done(mut self) {
        self.finish();
    }

    fn finish(&mut self) {
        if self.finished {
            return;
        }

        self.stop.store(true, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        if self.use_spinner {
            let mut stdout = io::stdout().lock();
            let _ = write!(stdout, "\r\x1B[2K\r");
            let _ = stdout.flush();
        }

        self.needs_clear.store(false, Ordering::SeqCst);
        self.finished = true;
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        self.finish();
    }
}
