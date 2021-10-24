use std::sync::Arc;

use flume::{Receiver, Sender};
use wgpu::{Surface, SurfaceError, SurfaceTexture};

pub struct AcquireThread {
    surface_sender: flume::Sender<Arc<Surface>>,
    frame_reciever: flume::Receiver<Result<SurfaceTexture, SurfaceError>>,
}

impl AcquireThread {
    pub fn new() -> Self {
        let (surface_sender, surface_reciever) = flume::unbounded();
        let (frame_sender, frame_reciever) = flume::unbounded();

        std::thread::spawn(move || Self::thread_loop(surface_reciever, frame_sender));

        Self {
            surface_sender,
            frame_reciever,
        }
    }

    pub async fn acquire(&self, surface: Arc<Surface>) -> Result<SurfaceTexture, SurfaceError> {
        self.surface_sender.send(surface).unwrap();
        self.frame_reciever.recv_async().await.unwrap()
    }

    fn thread_loop(
        surface_reciever: Receiver<Arc<Surface>>,
        frame_sender: Sender<Result<SurfaceTexture, SurfaceError>>,
    ) {
        profiling::register_thread!("rend3 acquire reactor");

        while let Ok(surface) = surface_reciever.recv() {
            let mut retrieved_texture = None;
            for _ in 0..10 {
                profiling::scope!("Inner Acquire Loop");
                match surface.get_current_texture() {
                    Ok(frame) => {
                        retrieved_texture = Some(Ok(frame));
                        break;
                    }
                    Err(SurfaceError::Timeout) => {}
                    Err(e) => retrieved_texture = Some(Err(e)),
                }
            }

            let tex = retrieved_texture.unwrap_or(Err(SurfaceError::Timeout));

            let send_e = frame_sender.send(tex).is_err();

            if send_e {
                break;
            }
        }
    }
}

impl Default for AcquireThread {
    fn default() -> Self {
        Self::new()
    }
}
