use bevy::mesh::{Mesh, MeshVertexAttribute, VertexAttributeValues};

/// Determines what kind of data each field in [`MeshExport`] carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[non_exhaustive]
pub enum GlyphMeta {
    /// Left to right count of the glyph, `0`, `1`, etc.
    #[default]
    Index,
    /// Returns x position in `em` of a vertex as if the text is rendered in a single line.
    Advance,
    /// Returns x position in `em` of the center of a glyph as if the text is rendered in a single line.
    PerGlyphAdvance,
    /// Returns a random value between `0..1` per glyph.
    RandomPerGlyph,
    /// Returns a random value between `0..1` per vertex.
    RandomPerVertex,
    /// The `uv.x` as if the text block is a rectangular sprite.
    UvX,
    /// The `uv.y` as if the text block is a rectangular sprite.
    UvY,
    /// Returns `0` for left corners and `1` for right corners.
    GlyphUvX,
    /// Returns `0` for bottom corners and `1` for top corners.
    GlyphUvY,
    /// Returns `0` for fill, `1` for stroke, `2` for shadow, `3` for image/emoji.
    Category,
    /// The [`SegmentStyle::magic_number`](crate::SegmentStyle::magic_number) field
    MagicNumber,
}

/// Determines what data to export as a part of the mesh.
#[derive(Debug, Clone, Default)]
pub enum MeshExport {
    /// Export nothing.
    #[default]
    None,
    /// Export two values as `uv1`.
    ///
    /// This does not require modification to the pipeline.
    Uv1(GlyphMeta, GlyphMeta),
    /// Export as an arbitrary amount of custom attributes.
    Custom(Vec<MeshExportEntry>),
}

impl MeshExport {
    pub fn init_cache(&self, mesh: &mut Mesh) -> Vec<MeshExportCache> {
        macro_rules! reuse {
            ($attr: expr, $data: ident) => {
                if let Some(VertexAttributeValues::$data(mut data)) = mesh.remove_attribute($attr) {
                    data.clear();
                    data
                } else {
                    Vec::new()
                }
            };
        }
        match self {
            MeshExport::None => Vec::new(),
            MeshExport::Uv1(x, y) => vec![MeshExportCache {
                entry: MeshExportEntry {
                    id: Mesh::ATTRIBUTE_UV_1,
                    len: 2,
                    meta: [*x, *y, *x, *y],
                },
                data: MeshExportCacheData::F2(reuse!(Mesh::ATTRIBUTE_UV_1, Float32x2)),
            }],
            MeshExport::Custom(items) => items
                .iter()
                .map(|x| MeshExportCache {
                    entry: *x,
                    data: match x.len {
                        1 => MeshExportCacheData::F1(reuse!(x.id, Float32)),
                        2 => MeshExportCacheData::F2(reuse!(x.id, Float32x2)),
                        3 => MeshExportCacheData::F3(reuse!(x.id, Float32x3)),
                        _ => MeshExportCacheData::F4(reuse!(x.id, Float32x4)),
                    },
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeshExportEntry {
    pub(crate) id: MeshVertexAttribute,
    pub(crate) len: usize,
    pub(crate) meta: [GlyphMeta; 4],
}

impl MeshExportEntry {
    /// Create a new export attribute.
    ///
    /// # Panics
    ///
    /// If attribute is not a float vector or if number of elements mismatches.
    pub fn new(entry: MeshVertexAttribute, meta: &[GlyphMeta]) -> Self {
        use bevy::mesh::VertexFormat::*;
        let len = meta.len().min(4);
        match entry.format {
            Float32 => assert_eq!(len, 1, "Expected 1 item."),
            Float32x2 => assert_eq!(len, 2, "Expected 2 items."),
            Float32x3 => assert_eq!(len, 3, "Expected 3 items."),
            Float32x4 => assert_eq!(len, 4, "Expected 4 items."),
            _ => panic!("Expected float vector."),
        };
        MeshExportEntry {
            id: entry,
            len: meta.len().min(4),
            meta: std::array::from_fn(|i| meta.get(i).copied().unwrap_or_default()),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, GlyphMeta)> + '_ {
        self.meta.iter().copied().enumerate().take(self.len)
    }
}
pub struct MeshExportCache {
    pub(crate) entry: MeshExportEntry,
    pub(crate) data: MeshExportCacheData,
}

pub enum MeshExportCacheData {
    F1(Vec<f32>),
    F2(Vec<[f32; 2]>),
    F3(Vec<[f32; 3]>),
    F4(Vec<[f32; 4]>),
}

impl MeshExportCacheData {
    pub fn extend_empty(&mut self) {
        match self {
            MeshExportCacheData::F1(items) => {
                items.extend([0.; 4]);
            }
            MeshExportCacheData::F2(items) => {
                items.extend([[0.; 2]; 4]);
            }
            MeshExportCacheData::F3(items) => {
                items.extend([[0.; 3]; 4]);
            }
            MeshExportCacheData::F4(items) => {
                items.extend([[0.; 4]; 4]);
            }
        }
    }

    pub fn with_inserted_quad(&mut self, index: usize, mut f: impl FnMut(usize, &mut f32)) {
        match self {
            MeshExportCacheData::F1(items) => {
                if let Some(chunk) = items.last_chunk_mut::<4>() {
                    chunk
                        .iter_mut()
                        .enumerate()
                        .for_each(|(vertex, item)| f(vertex, item))
                }
            }
            MeshExportCacheData::F2(items) => {
                if let Some(chunk) = items.last_chunk_mut::<4>() {
                    chunk.iter_mut().enumerate().for_each(|(vertex, item)| {
                        if let Some(item) = item.get_mut(index) {
                            f(vertex, item)
                        }
                    })
                }
            }
            MeshExportCacheData::F3(items) => {
                if let Some(chunk) = items.last_chunk_mut::<4>() {
                    chunk.iter_mut().enumerate().for_each(|(vertex, item)| {
                        if let Some(item) = item.get_mut(index) {
                            f(vertex, item)
                        }
                    })
                }
            }
            MeshExportCacheData::F4(items) => {
                if let Some(chunk) = items.last_chunk_mut::<4>() {
                    chunk.iter_mut().enumerate().for_each(|(vertex, item)| {
                        if let Some(item) = item.get_mut(index) {
                            f(vertex, item)
                        }
                    })
                }
            }
        }
    }

    pub fn for_each_zipped_mut<I: IntoIterator>(
        &mut self,
        joined: I,
        mut f: impl FnMut(&mut [f32], I::Item),
    ) {
        match self {
            MeshExportCacheData::F1(items) => {
                items
                    .iter_mut()
                    .map(|x| std::array::from_mut(x) as &mut [f32])
                    .zip(joined)
                    .for_each(|(a, b)| f(a, b));
            }
            MeshExportCacheData::F2(items) => {
                items
                    .iter_mut()
                    .map(|x| x as &mut [f32])
                    .zip(joined)
                    .for_each(|(a, b)| f(a, b));
            }
            MeshExportCacheData::F3(items) => {
                items
                    .iter_mut()
                    .map(|x| x as &mut [f32])
                    .zip(joined)
                    .for_each(|(a, b)| f(a, b));
            }
            MeshExportCacheData::F4(items) => {
                items
                    .iter_mut()
                    .map(|x| x as &mut [f32])
                    .zip(joined)
                    .for_each(|(a, b)| f(a, b));
            }
        }
    }
}

impl From<MeshExportCacheData> for VertexAttributeValues {
    fn from(val: MeshExportCacheData) -> Self {
        match val {
            MeshExportCacheData::F1(items) => VertexAttributeValues::Float32(items),
            MeshExportCacheData::F2(items) => VertexAttributeValues::Float32x2(items),
            MeshExportCacheData::F3(items) => VertexAttributeValues::Float32x3(items),
            MeshExportCacheData::F4(items) => VertexAttributeValues::Float32x4(items),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextMeshFaceCategory {
    Fill = 0,
    Stroke = 1,
    Shadow = 2,
    Image = 3,
}

impl TextMeshFaceCategory {
    /// Alias for [`TextMeshFaceCategory::Image`].
    #[allow(nonstandard_style)]
    pub const Emoji: Self = Self::Image;

    pub fn as_value(&self) -> f32 {
        (*self) as u8 as f32
    }
}
