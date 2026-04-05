use write_fonts::{
    read::tables::glyf::CurvePoint,
    tables::{
        cmap::{Cmap, CmapSubtable, EncodingRecord, PlatformId, SequentialMapGroup},
        glyf::{Bbox, Contour, GlyfLocaBuilder, SimpleGlyph},
        head::Head,
        hhea::Hhea,
        hmtx::Hmtx,
        maxp::Maxp,
        name::{Name, NameRecord},
        vmtx::LongMetric,
    },
    types::{FWord, NameId},
    OffsetMarker,
};

pub fn create_emoji_font(name: &str, widths: &[f32], c: char) -> Vec<u8> {
    use write_fonts::FontBuilder;

    let mut builder = FontBuilder::new();

    let _ = builder.add_table(&Name {
        name_record: vec![
            NameRecord::new(0, 0, 0, NameId::FAMILY_NAME, OffsetMarker::new(name.into())),
            NameRecord::new(
                0,
                0,
                0,
                NameId::SUBFAMILY_NAME,
                OffsetMarker::new("Regular".into()),
            ),
            NameRecord::new(0, 0, 0, NameId::UNIQUE_ID, OffsetMarker::new(name.into())),
            NameRecord::new(0, 0, 0, NameId::FULL_NAME, OffsetMarker::new(name.into())),
            NameRecord::new(
                0,
                0,
                0,
                NameId::POSTSCRIPT_NAME,
                OffsetMarker::new(name.into()),
            ),
            NameRecord::new(
                0,
                0,
                0,
                NameId::TYPOGRAPHIC_FAMILY_NAME,
                OffsetMarker::new(name.into()),
            ),
        ],
        lang_tag_record: None,
    });

    let _ = builder.add_table(&Head {
        units_per_em: 1000,
        ..Default::default()
    });

    let _ = builder.add_table(&Hhea {
        ascender: FWord::new(800),
        descender: FWord::new(-200),
        number_of_h_metrics: widths.len() as u16,
        ..Default::default()
    });

    let _ = builder.add_table(&Maxp {
        num_glyphs: widths.len() as u16,
        ..Default::default()
    });

    let _ = builder.add_table(&Hmtx {
        h_metrics: widths
            .iter()
            .copied()
            .map(|width| LongMetric::new((1000.0 * width) as u16, 0))
            .collect(),
        left_side_bearings: vec![],
    });

    let mut glyph_loca_builder = GlyfLocaBuilder::new();

    for width in widths {
        let x = 0;
        let y = -200;
        let w = (1000.0 * width) as i16;
        let h = 800;
        let _ = glyph_loca_builder.add_glyph(&SimpleGlyph {
            bbox: Bbox {
                x_min: x,
                y_min: y,
                x_max: w,
                y_max: h,
            },
            // needed because cosmic-text/fontdb currently ignores bbox
            contours: vec![Contour::from(vec![
                CurvePoint {
                    x,
                    y,
                    on_curve: true,
                },
                CurvePoint {
                    x: w,
                    y,
                    on_curve: true,
                },
                CurvePoint {
                    x: w,
                    y: h,
                    on_curve: true,
                },
                CurvePoint {
                    x,
                    y: h,
                    on_curve: true,
                },
            ])],
            instructions: vec![],
        });
    }

    let (glyf, loca, _) = glyph_loca_builder.build();

    let _ = builder.add_table(&glyf);
    let _ = builder.add_table(&loca);

    let _ = builder.add_table(&Cmap {
        encoding_records: vec![EncodingRecord {
            platform_id: PlatformId::Unicode,
            encoding_id: 1,
            subtable: OffsetMarker::new(CmapSubtable::format_12(
                0,
                vec![SequentialMapGroup {
                    start_char_code: c as u32,
                    end_char_code: c as u32 + widths.len() as u32 - 1,
                    start_glyph_id: 0,
                }],
            )),
        }],
    });

    builder.build()
}
