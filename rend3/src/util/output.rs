use std::sync::Arc;
use wgpu::{SurfaceError, SurfaceTexture, TextureView, TextureViewDescriptor};

use crate::{types::Surface, util::acquire::AcquireThread};

pub enum OutputFrame {
    // A surface which has not yet been acquired. This lets rend3 acquire as late as possible.
    Surface {
        surface: Arc<Surface>,
    },
    // Pre-acquired surface. rend3 will present it.
    SurfaceAcquired {
        view: TextureView,
        surface_tex: SurfaceTexture,
    },
    // Arbitrary texture view.
    View(Arc<TextureView>),
}

impl OutputFrame {
    pub async fn acquire(&mut self, acquire: &AcquireThread) -> Result<(), SurfaceError> {
        if let Self::Surface { surface } = self {
            let surface_tex = acquire.acquire(Arc::clone(surface)).await?;

            let view = surface_tex.texture.create_view(&TextureViewDescriptor::default());

            *self = Self::SurfaceAcquired { view, surface_tex }
        }

        Ok(())
    }

    pub fn as_view(&self) -> Option<&TextureView> {
        match self {
            Self::Surface { .. } => None,
            Self::SurfaceAcquired { view, .. } => Some(view),
            Self::View(inner) => Some(&**inner),
        }
    }

    pub fn present(self) {
        if let Self::SurfaceAcquired {
            surface_tex: surface, ..
        } = self
        {
            surface.present();
        }
    }
}
