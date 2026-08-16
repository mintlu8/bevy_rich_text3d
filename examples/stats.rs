use bevy::{
    app::{App, PostStartup, Startup, Update},
    asset::Assets,
    color::{Color, Srgba},
    ecs::query::With,
    input::{keyboard::KeyCode, ButtonInput},
    light::GlobalAmbientLight,
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Component, Entity, Local, Mesh3d, OrthographicProjection,
        Projection, Query, Res, ResMut, Resource, Transform,
    },
    time::{Time, Virtual},
    DefaultPlugins,
};
use bevy_rich_text3d::{
    ConditionOutput, FetchTextPlugin, FetchedCondition, ParseBuilder, ParseError, SegmentStyle,
    SharedSegment, Text3d, Text3dBounds, Text3dPlugin, Text3dSegment, Text3dStyle, TextAlign,
    TextAnchor, TextAtlas, TextFetch,
};
use rustc_hash::FxHashMap;
use std::str::FromStr;

#[derive(Debug, Component)]
pub struct Unit(&'static str);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Stat {
    Strength,
    Intellect,
    Agility,
    Defense,
    Stamina,
}

impl FromStr for Stat {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "strength" => Stat::Strength,
            "intellect" => Stat::Intellect,
            "agility" => Stat::Agility,
            "defense" => Stat::Defense,
            "stamina" => Stat::Stamina,
            s => return Err(ParseError::BadCommand(format!("Unknown stat {s}."))),
        })
    }
}

#[derive(Debug, Component)]
pub struct StatMap(FxHashMap<Stat, i32>);

#[derive(Debug, Resource)]
pub struct NameToUnit(FxHashMap<String, Entity>);

pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        })
        .add_plugins(FetchTextPlugin)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        });
    app.world_mut().spawn((
        Unit("Samuel"),
        StatMap(FxHashMap::from_iter([
            (Stat::Strength, 1),
            (Stat::Intellect, 2),
            (Stat::Agility, 3),
            (Stat::Defense, 4),
            (Stat::Stamina, 5),
        ])),
    ));
    app.world_mut().spawn((
        Unit("Catalina"),
        StatMap(FxHashMap::from_iter([
            (Stat::Strength, 5),
            (Stat::Intellect, 5),
            (Stat::Agility, 5),
            (Stat::Defense, 5),
            (Stat::Stamina, 5),
        ])),
    ));
    app.world_mut().spawn((
        Unit("Rufus"),
        StatMap(FxHashMap::from_iter([
            (Stat::Strength, 5),
            (Stat::Intellect, 5),
            (Stat::Agility, 5),
            (Stat::Defense, 5),
            (Stat::Stamina, 5),
        ])),
    ));

    app.add_systems(
        Startup,
        |mut commands: Commands, units: Query<(Entity, &Unit)>| {
            commands.insert_resource(NameToUnit(
                units.iter().map(|(e, n)| (n.0.to_owned(), e)).collect(),
            ));
        },
    );
    let shift_pressed = app
        .world_mut()
        .spawn((SharedSegment, ShiftPressed, FetchedCondition(false)))
        .id();

    app.add_systems(PostStartup, move |mut commands: Commands, name_to_unit: Res<NameToUnit>, mut standard_materials: ResMut<Assets<StandardMaterial>>| {
            let mat = standard_materials.add(
                StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..Default::default()
                }
            );
            let mut parse = |s: &str| {
                let vec: Vec<_> = s.split('.').collect();
                if let [name, stat] = vec.as_slice() {
                    let (stat, add) = match stat.split_once('+') {
                        Some((stat, plus)) => (stat, plus.parse::<i32>().unwrap_or(0)),
                        None => (*stat, 0),
                    };

                    let stat = Stat::from_str(stat)?;
                    let unit = *name_to_unit.0.get(*name)
                        .ok_or(ParseError::Custom(format!("Unknown unit {name}.")))?;
                    Ok((Text3dSegment::Extract(
                        commands.spawn(TextFetch::fetch_component::<StatMap>(unit, move |map| {
                            (map.0.get(&stat).copied().unwrap_or_default() + add).to_string()
                        })).id()
                    ), SegmentStyle::default()))
                } else {
                    Err(ParseError::Custom("".to_owned()))
                }
            };
            let parse_condition = |s: &str| {
                if s == "shift" {
                    Ok(ConditionOutput::Dynamic(shift_pressed))
                } else {
                    Err(ParseError::Custom("".to_owned()))
                }
            };
            let text1 = Text3d::parse(
                "**Samuel**\nStrength: {Samuel.strength}\nIntellect: {Samuel.intellect}\nAgility: {Samuel.agility}\nDefense: {Samuel.defense}\nStamina: {Samuel.stamina}", 
                ParseBuilder::new().with_parse_value(&mut parse).with_parse_condition(parse_condition)
            ).unwrap();
            let text2 = Text3d::parse(
                "**Catalina**\nStrength: {Catalina.strength}\nIntellect: {Catalina.intellect}\nAgility: {Catalina.agility}\nDefense: {Catalina.defense}\nStamina: {Catalina.stamina}", 
                ParseBuilder::new().with_parse_value(&mut parse).with_parse_condition(parse_condition)
            ).unwrap();
            let text3 = Text3d::parse(
                "Samuel's Sundering Blade:\n Deals {?shift:strength({Samuel.strength}) + 4}{?!shift:{Samuel.strength+4}} damage.", 
                ParseBuilder::new().with_parse_value(&mut parse).with_parse_condition(parse_condition)
            ).unwrap();
            let text4 = Text3d::parse(
                "Catalina's Fire Bolt:\n Deals {?shift:intellect({Catalina.intellect}) + 12}{?!shift:{Catalina.intellect+12}} damage.", 
                ParseBuilder::new().with_parse_value(&mut parse).with_parse_condition(parse_condition)
            ).unwrap();
            commands.spawn((
                text1,
                Text3dStyle {
                    size: 32.,
                    color: Srgba::new(0., 1., 1., 1.),
                    align: TextAlign::Center,
                    anchor: TextAnchor::CENTER_LEFT,
                    ..Default::default()
                },
                Text3dBounds {
                    width: 400.,
                },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
            ));

            commands.spawn((
                text2,
                Text3dStyle {
                    size: 32.,
                    color: Srgba::new(1., 0., 0., 1.),
                    align: TextAlign::Center,
                    anchor: TextAnchor::CENTER_RIGHT,
                    ..Default::default()
                },
                Text3dBounds {
                    width: 400.,
                },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
            ));

            commands.spawn((
                text3,
                Text3dStyle {
                    size: 32.,
                    color: Srgba::new(0., 1., 1., 1.),
                    align: TextAlign::Center,
                    ..Default::default()
                },
                Text3dBounds {
                    width: 600.,
                },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(Vec3::new(0.0, -200.0, 0.0))
            ));

            commands.spawn((
                text4,
                Text3dStyle {
                    size: 32.,
                    color: Srgba::new(1., 0., 0., 1.),
                    align: TextAlign::Center,
                    ..Default::default()
                },
                Text3dBounds {
                    width: 600.,
                },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(Vec3::new(0.0, -300.0, 0.0))
            ));

            commands.spawn((
                Camera3d::default(),
                Projection::Orthographic(OrthographicProjection::default_3d()),
                Transform::from_translation(Vec3::new(0., 0., 1.))
                    .looking_at(Vec3::new(0., 0., 0.), Vec3::Y)
            ));
        });
    app.add_systems(Update, randomize_stats);
    app.add_systems(Update, check_shift);
    app.run();
}

fn randomize_stats(
    mut timer: Local<f32>,
    time: Res<Time<Virtual>>,
    mut query: Query<&mut StatMap>,
) {
    *timer += time.delta_secs();
    if *timer > 5.0 {
        *timer -= 5.0;

        for mut stats in &mut query {
            stats
                .0
                .iter_mut()
                .for_each(|(_, v)| *v = fastrand::i32(0..10));
        }
    }
}

#[derive(Debug, Component)]
struct ShiftPressed;

fn check_shift(
    presses: ResMut<ButtonInput<KeyCode>>,
    query: Query<&mut FetchedCondition, With<ShiftPressed>>,
) {
    if presses.pressed(KeyCode::ShiftLeft) {
        for mut item in query {
            if !item.0 {
                item.0 = true;
            }
        }
    } else {
        for mut item in query {
            if item.0 {
                item.0 = false;
            }
        }
    }
}
