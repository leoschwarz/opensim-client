use data::Uuid;
use std::sync::Arc;

// TODO:
// - In memory cache for textures, limiting the amount of memory used for textures kept in cache.
// - Or should textures not be kept in main memory at all, and just the GPU memory should be
// managed.

pub struct Texture {
    /// The texture id on the sim (or grid?).
    id: Uuid,

    handle: Arc<TextureHandle>,
}

impl Texture {

}

pub struct TextureHandle {

}
