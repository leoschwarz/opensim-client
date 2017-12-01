//! This contains everything to manage information about the terrain.

use data::PatchLocator;
use nalgebra::MatrixN;

pub type PatchMatrix<S> = MatrixN<S, ::typenum::U256>;

pub struct TerrainManager {

}

impl TerrainManager {
    pub fn new() -> Self {
        TerrainManager {}
    }

    pub fn get_patch(&mut self, loc: &PatchLocator) -> Result<TerrainDataPatch, ()>
    {
        let data = include!("layer_land.png.txt");

        // TODO: dummy implementation
        Ok(TerrainDataPatch {
            land: PatchMatrix::from_fn(|x, y| {
                data[x][y]
            })
        })
    }
}

/// The decoded terrain data for a 256x256m patch.
pub struct TerrainDataPatch {
    /// The land heightmap.
    pub land: PatchMatrix<f32>,
}
