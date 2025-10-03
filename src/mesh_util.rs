use bevy::{
    color::{ColorToComponents, LinearRgba, Srgba},
    image::Image,
    math::{Rect, Vec2},
    mesh::{Indices, Mesh, VertexAttributeValues},
};

use crate::{layers::Layer, GlyphMeta, Text3dStyling};

// Take the allocation if possible but clear the data.
macro_rules! recycle_mesh {
    ($mesh: expr, $attr: ident, $ty: ident) => {
        if let Some(VertexAttributeValues::$ty(mut v)) = $mesh.remove_attribute(Mesh::$attr) {
            v.clear();
            v
        } else {
            Vec::new()
        }
    };
}

fn corners_z(rect: Rect, z: f32) -> [[f32; 3]; 4] {
    [
        [rect.min.x, rect.min.y, z],
        [rect.max.x, rect.min.y, z],
        [rect.min.x, rect.max.y, z],
        [rect.max.x, rect.max.y, z],
    ]
}

fn corners(rect: Rect) -> [[f32; 2]; 4] {
    [
        [rect.min.x, rect.min.y],
        [rect.max.x, rect.min.y],
        [rect.min.x, rect.max.y],
        [rect.max.x, rect.max.y],
    ]
}

pub(crate) struct ExtractedMesh<'t> {
    pub mesh: &'t mut Mesh,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uv0: Vec<[f32; 2]>,
    pub uv1: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u16>,
    pub sort: &'t mut Vec<(Layer, [u16; 6])>,
    pub layer_offset: f32,
}

impl<'t> ExtractedMesh<'t> {
    pub fn new(
        mesh: &'t mut Mesh,
        sort_buffer: &'t mut Vec<(Layer, [u16; 6])>,
        layer_offset: f32,
    ) -> Self {
        sort_buffer.clear();
        let positions = recycle_mesh!(mesh, ATTRIBUTE_POSITION, Float32x3);
        let normals = recycle_mesh!(mesh, ATTRIBUTE_NORMAL, Float32x3);
        let uv0 = recycle_mesh!(mesh, ATTRIBUTE_UV_0, Float32x2);
        let uv1 = recycle_mesh!(mesh, ATTRIBUTE_UV_1, Float32x2);
        let colors = recycle_mesh!(mesh, ATTRIBUTE_COLOR, Float32x4);

        let mut indices = if let Some(Indices::U16(indices)) = mesh.remove_indices() {
            indices
        } else {
            Vec::new()
        };
        indices.clear();
        ExtractedMesh {
            mesh,
            positions,
            normals,
            uv0,
            uv1,
            colors,
            indices,
            sort: sort_buffer,
            layer_offset,
        }
    }

    pub fn pixel_to_uv(&mut self, image: &Image) {
        let inv_width = 1.0 / image.width() as f32;
        let inv_height = 1.0 / image.height() as f32;

        self.uv0.iter_mut().for_each(|[x, y]| {
            *x *= inv_width;
            *y *= inv_height;
        });
    }

    pub fn post_process_uv1(&mut self, styling: &Text3dStyling, min: Vec2, dimension: Vec2) {
        for (meta_type, i) in [(styling.uv1.0, 0), (styling.uv1.1, 1)] {
            match meta_type {
                GlyphMeta::RowX => {
                    for (uv1, position) in self.uv1.iter_mut().zip(self.positions.iter()) {
                        uv1[i] = (position[0] - min.x) / dimension.x;
                    }
                }
                GlyphMeta::ColY => {
                    for (uv1, position) in self.uv1.iter_mut().zip(self.positions.iter()) {
                        uv1[i] = (position[1] - min.y) / dimension.y;
                    }
                }
                _ => (),
            }
        }
    }

    pub fn translate(&mut self, mut f: impl FnMut(&mut Vec2)) {
        for [x, y, _] in &mut self.positions {
            let mut v = Vec2::new(*x, *y);
            f(&mut v);
            *x = v.x;
            *y = v.y;
        }
    }

    pub fn cache_rectangle(
        &mut self,
        base: Vec2,
        texture: Rect,
        color: Srgba,
        scale_factor: f32,
        layer: Layer,
        real_index: usize,
        advance: f32,
        magic_number: f32,
        styling: &Text3dStyling,
    ) {
        let mesh_rect = Rect {
            min: base,
            max: base + texture.size() / scale_factor,
        };
        self.cache_rectangle2(
            mesh_rect,
            texture,
            color,
            layer,
            real_index,
            advance,
            magic_number,
            styling,
        );
    }

    pub fn cache_rectangle2(
        &mut self,
        mesh_rect: Rect,
        texture: Rect,
        color: Srgba,
        layer: Layer,
        real_index: usize,
        advance: f32,
        magic_number: f32,
        styling: &Text3dStyling,
    ) {
        let i = self.positions.len() as u16;
        self.sort
            .push((layer, [i, i + 1, i + 2, i + 1, i + 3, i + 2]));

        self.positions.extend(corners_z(mesh_rect, 0.));
        self.normals.extend([[0., 0., 1.]; 4]);
        self.colors
            .extend([LinearRgba::from(color).to_f32_array(); 4]);

        // First we cache the pixel position since the texture may be resized.
        self.uv0.extend(corners(texture));

        let mut uv1_buffer = [[0., 0.], [0., 0.], [0., 0.], [0., 0.]];

        for (meta_type, i) in [(styling.uv1.0, 0), (styling.uv1.1, 1)] {
            match meta_type {
                GlyphMeta::Index => {
                    for pair in &mut uv1_buffer {
                        pair[i] = real_index as f32;
                    }
                }
                GlyphMeta::Advance => {
                    let x0 = advance / styling.size;
                    let x1 = (advance + mesh_rect.width()) / styling.size;
                    uv1_buffer[0][i] = x0;
                    uv1_buffer[1][i] = x1;
                    uv1_buffer[2][i] = x0;
                    uv1_buffer[3][i] = x1;
                }
                GlyphMeta::PerGlyphAdvance => {
                    let x = (advance + mesh_rect.width() / 2.0) / styling.size;
                    uv1_buffer[0][i] = x;
                    uv1_buffer[1][i] = x;
                    uv1_buffer[2][i] = x;
                    uv1_buffer[3][i] = x;
                }
                GlyphMeta::MagicNumber => {
                    uv1_buffer[0][i] = magic_number;
                    uv1_buffer[1][i] = magic_number;
                    uv1_buffer[2][i] = magic_number;
                    uv1_buffer[3][i] = magic_number;
                }
                GlyphMeta::RowX => (),
                GlyphMeta::ColY => (),
            }
        }

        self.uv1.extend(uv1_buffer);
    }
}

impl Drop for ExtractedMesh<'_> {
    fn drop(&mut self) {
        use std::mem::take;
        self.sort.sort_by_key(|x| x.0);
        if self.layer_offset != 0.0 {
            let mut offset = 0.0;
            let mut layer = self.sort.last().map(|x| x.0).unwrap_or(Layer::None);
            for (l, entry) in self.sort.iter().rev() {
                if layer != *l {
                    offset -= self.layer_offset;
                    layer = *l;
                }
                for idx in entry {
                    if let Some([_, _, z]) = self.positions.get_mut(*idx as usize) {
                        *z = offset;
                    }
                }
            }
        }
        self.indices
            .extend(self.sort.drain(..).flat_map(|(_, v)| v));
        if !self.positions.is_empty() {
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_POSITION, take(&mut self.positions));
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_NORMAL, take(&mut self.normals));
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_COLOR, take(&mut self.colors));
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_UV_0, take(&mut self.uv0));
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_UV_1, take(&mut self.uv1));
            self.mesh
                .insert_indices(Indices::U16(take(&mut self.indices)));
        } else {
            // Placeholder, since empty mesh panics on some platforms.
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
            self.mesh
                .insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.0, 0.0]; 3]);
            self.mesh.insert_indices(Indices::U16(vec![0, 1, 2]));
        }
    }
}
