use flax::{component, Debuggable};

use crate::{
    assets::Handle,
    wgpu::{
        font::{Font, FontFromFile},
        graphics::texture::Texture,
        shape_renderer::DrawCommand,
    },
};

use super::mesh_buffer::MeshHandle;

component! {
    /// The gpu texture to use for rendering
    pub(crate) texture: Handle<Texture>,

    pub(crate) font: Handle<Font>,

    pub font_from_file: FontFromFile => [ Debuggable ],

    /// Renderer specific data for drawing a shape
    pub(crate) draw_cmd: DrawCommand => [ Debuggable ],

    pub(crate) mesh_handle: MeshHandle => [ Debuggable ],

    pub model_matrix: glam::Mat4 => [ Debuggable ],
}
