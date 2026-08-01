mod map;

use adam_core::{Money, ProgramFundingSource, World, WorldCommand};
use adam_observatory::{ObservatorySnapshot, RegionSnapshot};
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::text::Font;
use bevy::window::WindowResolution;

use map::{BorderClass, MapCell, Relief, StrategicMap};

/// The interface font is embedded in the executable so Cyrillic text never
/// depends on an `assets` folder next to the binary or on system fonts.
const INTERFACE_FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
const MAP_SEED: u64 = 0x0ADA_1234_5678_9ABC;

const OCEAN_DEEP: Color = Color::srgb(0.043, 0.070, 0.110);
const OCEAN_SHELF: Color = Color::srgb(0.072, 0.113, 0.168);

#[derive(Resource)]
struct Observatory(ObservatorySnapshot);

#[derive(Resource)]
struct Simulation(World);

#[derive(Resource)]
struct MapGeometry(StrategicMap);

#[derive(Resource)]
struct InterfaceFont(Handle<Font>);

#[derive(Resource)]
struct ProgramDeskState {
    message: String,
}

#[derive(Resource)]
struct TimeFlow {
    paused: bool,
    speed: u8,
    accumulated_hours: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum OverlayMode {
    Political,
    Confidence,
    Population,
    Output,
}

impl OverlayMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Political => "Политическая",
            Self::Confidence => "Доверие",
            Self::Population => "Население",
            Self::Output => "Производство",
        }
    }
}

#[derive(Resource)]
struct InspectorState {
    selected: usize,
    overlay: OverlayMode,
}

#[derive(Resource, Default)]
struct MapHover {
    cell: Option<usize>,
    region: Option<usize>,
    screen_position: Vec2,
}

#[derive(Component)]
struct ProvinceCell {
    cell: usize,
    region: usize,
}

#[derive(Component)]
struct BorderLine {
    class: BorderClass,
    left: Option<usize>,
    right: Option<usize>,
}

#[derive(Component)]
struct InspectorText;

#[derive(Component)]
struct OverlayText;

#[derive(Component)]
struct ProgramDeskText;

#[derive(Component)]
struct TimelineText;

#[derive(Component)]
struct DateText;

#[derive(Component)]
struct LegendText;

#[derive(Component)]
struct MapTooltipText;

#[derive(Component)]
struct MapCamera;

#[derive(Component)]
struct NationalStatusText;

#[derive(Component)]
struct RegionLabel;

#[derive(Component)]
struct CountryLabel;

#[derive(Component)]
struct ReliefMarker;

#[derive(Component)]
struct CapitalMarker;

#[derive(Clone, Copy, Component)]
enum UiCommandButton {
    Map(OverlayMode),
    Pause,
    Speed(u8),
}

#[derive(Clone, Copy, Component)]
enum MetricBar {
    Confidence,
    Population,
    Output,
}

#[derive(Component)]
struct CountryFlag;

fn main() {
    let world = adam_content::observatory_world(1).expect("observatory world must load");
    let snapshot = ObservatorySnapshot::capture(&world);
    let ownership: Vec<u32> = snapshot
        .regions
        .iter()
        .map(|region| region.country.get())
        .collect();
    let geometry = StrategicMap::generate(&ownership, MAP_SEED);
    App::new()
        .insert_resource(Observatory(snapshot))
        .insert_resource(Simulation(world))
        .insert_resource(MapGeometry(geometry))
        .insert_resource(ProgramDeskState {
            message: "Программа объявлена. Выберите источник финансирования.".to_owned(),
        })
        .insert_resource(TimeFlow {
            paused: false,
            speed: 1,
            accumulated_hours: 0.0,
        })
        .insert_resource(InspectorState {
            selected: 0,
            overlay: OverlayMode::Political,
        })
        .insert_resource(MapHover::default())
        .insert_resource(ClearColor(OCEAN_DEEP))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "A.D.A.M — государственная стратегическая обсерватория".to_owned(),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .add_systems(PreStartup, load_interface_font)
        .add_systems(Startup, setup_observatory)
        .add_systems(
            Update,
            (
                apply_interface_font,
                handle_ui_buttons,
                handle_observatory_input,
                update_map_pointer,
                control_map_camera,
                update_map_lod,
                advance_game_clock,
                refresh_header,
                refresh_ui_buttons,
                refresh_map_colors,
                refresh_inspector_panels,
                refresh_metric_bars,
                refresh_national_status,
                refresh_program_desk,
                refresh_timeline,
            )
                .chain(),
        )
        .run();
}

fn load_interface_font(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = Font::try_from_bytes(INTERFACE_FONT_BYTES.to_vec())
        .expect("bundled interface font must parse");
    commands.insert_resource(InterfaceFont(fonts.add(font)));
}

/// Every text spawned by the observatory adopts the bundled Cyrillic face,
/// including text created after startup such as map tooltips.
#[allow(clippy::needless_pass_by_value)]
fn apply_interface_font(
    font: Res<InterfaceFont>,
    mut text_fonts: Query<&mut TextFont, Added<TextFont>>,
) {
    for mut text_font in &mut text_fonts {
        text_font.font = font.0.clone();
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn setup_observatory(
    mut commands: Commands,
    observatory: Res<Observatory>,
    geometry: Res<MapGeometry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Transform::from_scale(Vec3::splat(1.35)),
        MapCamera,
    ));
    spawn_strategic_map(
        &mut commands,
        &mut meshes,
        &mut materials,
        &geometry.0,
        &observatory.0,
    );
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            height: Val::Px(88.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.055, 0.090, 0.97)),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(100.0),
            width: Val::Px(224.0),
            height: Val::Px(290.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.030, 0.050, 0.078, 0.94)),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            top: Val::Px(92.0),
            width: Val::Px(330.0),
            height: Val::Px(288.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.045, 0.070, 0.105, 0.94)),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(16.0),
            width: Val::Px(770.0),
            height: Val::Px(235.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.050, 0.080, 0.95)),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            bottom: Val::Px(16.0),
            width: Val::Px(400.0),
            height: Val::Px(300.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.040, 0.075, 0.070, 0.96)),
    ));
    commands.spawn((
        Text::new(header_text(&observatory.0, false, 1)),
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::srgb(0.91, 0.94, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(24.0),
            top: Val::Px(18.0),
            ..default()
        },
        DateText,
    ));
    commands.spawn((
        Text::new("A.D.A.M"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.94, 0.77, 0.34)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("ГОСУДАРСТВЕННАЯ ОБСЕРВАТОРИЯ  /  СТРАТЕГИЧЕСКОЕ УПРАВЛЕНИЕ"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.55, 0.66, 0.79)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(26.0),
            top: Val::Px(51.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new(national_status_text(&observatory.0, 0)),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.73, 0.82, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(390.0),
            top: Val::Px(51.0),
            width: Val::Px(500.0),
            ..default()
        },
        NationalStatusText,
    ));
    commands.spawn((
        Text::new("СЛОИ КАРТЫ  /  ПОЛИТИЧЕСКАЯ"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.63, 0.72, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(30.0),
            top: Val::Px(112.0),
            ..default()
        },
        OverlayText,
    ));
    commands.spawn((
        Text::new(legend_text(OverlayMode::Political)),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(0.57, 0.69, 0.79)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(30.0),
            top: Val::Px(280.0),
            width: Val::Px(200.0),
            ..default()
        },
        LegendText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.94, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(240.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.92)),
        Visibility::Hidden,
        MapTooltipText,
    ));
    commands.spawn((
        Text::new(inspector_text(observatory.0.regions.first())),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.90, 0.70)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(24.0),
            top: Val::Px(90.0),
            width: Val::Px(230.0),
            ..default()
        },
        InspectorText,
    ));
    commands.spawn((
        Text::new(timeline_text(&observatory.0)),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.76, 0.80, 0.90)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            bottom: Val::Px(24.0),
            width: Val::Px(760.0),
            ..default()
        },
        TimelineText,
    ));
    commands.spawn((
        Text::new("ГОСУДАРСТВЕННАЯ ПРОГРАММА"),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::srgb(0.75, 0.90, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(24.0),
            bottom: Val::Px(28.0),
            width: Val::Px(380.0),
            ..default()
        },
        ProgramDeskText,
    ));
    spawn_command_button(
        &mut commands,
        30.0,
        142.0,
        190.0,
        "1   Политическая",
        UiCommandButton::Map(OverlayMode::Political),
    );
    spawn_command_button(
        &mut commands,
        30.0,
        174.0,
        190.0,
        "2   Доверие",
        UiCommandButton::Map(OverlayMode::Confidence),
    );
    spawn_command_button(
        &mut commands,
        30.0,
        206.0,
        190.0,
        "3   Население",
        UiCommandButton::Map(OverlayMode::Population),
    );
    spawn_command_button(
        &mut commands,
        30.0,
        238.0,
        190.0,
        "4   Производство",
        UiCommandButton::Map(OverlayMode::Output),
    );
    spawn_command_button(
        &mut commands,
        900.0,
        51.0,
        72.0,
        "ПАУЗА",
        UiCommandButton::Pause,
    );
    for (index, speed) in [1_u8, 4, 12, 24, 72].into_iter().enumerate() {
        let index = index as f32;
        spawn_command_button(
            &mut commands,
            978.0 + index * 52.0,
            51.0,
            46.0,
            &speed.to_string(),
            UiCommandButton::Speed(speed),
        );
    }
    spawn_country_flag(&mut commands);
    spawn_metric_bar(
        &mut commands,
        246.0,
        "ДОВЕРИЕ",
        MetricBar::Confidence,
        Color::srgb(0.35, 0.74, 0.55),
    );
    spawn_metric_bar(
        &mut commands,
        278.0,
        "НАСЕЛЕНИЕ",
        MetricBar::Population,
        Color::srgb(0.35, 0.61, 0.86),
    );
    spawn_metric_bar(
        &mut commands,
        310.0,
        "ПРОИЗВОДСТВО",
        MetricBar::Output,
        Color::srgb(0.88, 0.68, 0.26),
    );
}

#[allow(clippy::too_many_lines)]
fn spawn_strategic_map(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    geometry: &StrategicMap,
    snapshot: &ObservatorySnapshot,
) {
    commands.spawn((
        Sprite::from_color(OCEAN_DEEP, geometry.half_extent * 2.0 + Vec2::splat(1200.0)),
        Transform::from_xyz(0.0, 0.0, -40.0),
    ));
    let grid_step = map::HEX_RADIUS * 4.0;
    let mut latitude = -geometry.half_extent.y;
    while latitude <= geometry.half_extent.y {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.30, 0.45, 0.55, 0.055),
                Vec2::new(geometry.half_extent.x * 2.0, 1.0),
            ),
            Transform::from_xyz(0.0, latitude, -30.0),
        ));
        latitude += grid_step;
    }
    let mut longitude = -geometry.half_extent.x;
    while longitude <= geometry.half_extent.x {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.30, 0.45, 0.55, 0.055),
                Vec2::new(1.0, geometry.half_extent.y * 2.0),
            ),
            Transform::from_xyz(longitude, 0.0, -30.0),
        ));
        longitude += grid_step;
    }

    for (index, cell) in geometry.cells.iter().enumerate() {
        let mesh = meshes.add(polygon_mesh(cell));
        let material = materials.add(ColorMaterial {
            color: cell_color(cell, snapshot, OverlayMode::Political),
            ..default()
        });
        let depth = if cell.region().is_some() { 0.0 } else { -20.0 };
        let mut entity = commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, depth),
        ));
        if let Some(region) = cell.region() {
            entity.insert(ProvinceCell {
                cell: index,
                region,
            });
            spawn_relief(commands, cell);
        }
    }

    for border in &geometry.borders {
        let delta = border.to - border.from;
        let length = delta.length();
        if length <= f32::EPSILON {
            continue;
        }
        let (width, color, depth) = border_style(border.class);
        commands.spawn((
            Sprite::from_color(color, Vec2::new(length + width * 0.6, width)),
            Transform {
                translation: ((border.from + border.to) * 0.5).extend(depth),
                rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
                ..default()
            },
            BorderLine {
                class: border.class,
                left: border.left,
                right: border.right,
            },
        ));
    }

    for (region, position) in geometry.region_capitals.iter().enumerate() {
        if geometry
            .region_cell_counts
            .get(region)
            .copied()
            .unwrap_or_default()
            == 0
        {
            continue;
        }
        commands.spawn((
            Sprite::from_color(Color::srgb(0.05, 0.06, 0.08), Vec2::splat(13.0)),
            Transform::from_xyz(position.x, position.y, 3.0),
            CapitalMarker,
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.96, 0.80, 0.35), Vec2::splat(8.0)),
            Transform {
                translation: position.extend(3.1),
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                ..default()
            },
            CapitalMarker,
        ));
    }

    for (region, centroid) in geometry.region_centroids.iter().enumerate() {
        let Some(snapshot_region) = snapshot.regions.get(region) else {
            continue;
        };
        commands.spawn((
            Text2d::new(localized_region_name(&snapshot_region.name).to_uppercase()),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgba(0.96, 0.97, 0.94, 0.86)),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_xyz(centroid.x, centroid.y - 26.0, 4.0),
            RegionLabel,
        ));
    }

    for (country, centroid) in country_label_anchors(geometry, snapshot) {
        commands.spawn((
            Text2d::new(spaced_caps(localized_country_name(country))),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::srgba(0.97, 0.97, 0.90, 0.30)),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_xyz(centroid.x, centroid.y, 2.0),
            CountryLabel,
        ));
    }
}

fn spawn_relief(commands: &mut Commands, cell: &MapCell) {
    let (size, color, count) = match cell.relief {
        Relief::Mountains => (7.0, Color::srgba(0.82, 0.84, 0.88, 0.45), 2),
        Relief::Hills => (5.0, Color::srgba(0.74, 0.72, 0.60, 0.32), 1),
        Relief::Forest => (4.5, Color::srgba(0.12, 0.26, 0.16, 0.45), 3),
        Relief::Plain => return,
    };
    for step in 0..count {
        let offset = Vec2::new(
            (step as f32 - (count as f32 - 1.0) * 0.5) * (size + 3.0),
            (step % 2) as f32 * 2.5 - 1.0,
        );
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(size)),
            Transform {
                translation: (cell.center + offset).extend(1.5),
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                ..default()
            },
            ReliefMarker,
        ));
    }
}

fn polygon_mesh(cell: &MapCell) -> Mesh {
    let mut positions = Vec::with_capacity(7);
    let mut uvs = Vec::with_capacity(7);
    positions.push([cell.center.x, cell.center.y, 0.0]);
    uvs.push([0.5, 0.5]);
    for corner in cell.polygon {
        positions.push([corner.x, corner.y, 0.0]);
        let local = (corner - cell.center) / (map::HEX_RADIUS * 2.0) + Vec2::splat(0.5);
        uvs.push([local.x, local.y]);
    }
    let mut indices = Vec::with_capacity(18);
    for corner in 0..6_u32 {
        indices.extend_from_slice(&[0, corner + 1, (corner + 1) % 6 + 1]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

const fn border_style(class: BorderClass) -> (f32, Color, f32) {
    match class {
        BorderClass::Province => (1.3, Color::srgba(0.03, 0.04, 0.05, 0.30), 0.6),
        BorderClass::Coast => (2.4, Color::srgba(0.36, 0.58, 0.66, 0.55), 0.8),
        BorderClass::Region => (2.4, Color::srgba(0.02, 0.03, 0.04, 0.80), 1.0),
        BorderClass::Country => (4.2, Color::srgba(0.03, 0.03, 0.05, 0.95), 1.2),
    }
}

fn country_label_anchors(
    geometry: &StrategicMap,
    snapshot: &ObservatorySnapshot,
) -> Vec<(u32, Vec2)> {
    let mut anchors: Vec<(u32, Vec2, f32)> = Vec::new();
    for (region, centroid) in geometry.region_centroids.iter().enumerate() {
        let Some(snapshot_region) = snapshot.regions.get(region) else {
            continue;
        };
        let weight = geometry
            .region_cell_counts
            .get(region)
            .copied()
            .unwrap_or_default() as f32;
        if weight <= 0.0 {
            continue;
        }
        let country = snapshot_region.country.get();
        if let Some(entry) = anchors.iter_mut().find(|entry| entry.0 == country) {
            entry.1 += *centroid * weight;
            entry.2 += weight;
        } else {
            anchors.push((country, *centroid * weight, weight));
        }
    }
    anchors
        .into_iter()
        .map(|(country, sum, weight)| (country, sum / weight))
        .collect()
}

fn spaced_caps(text: &str) -> String {
    text.to_uppercase()
        .chars()
        .map(|character| character.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn spawn_command_button(
    commands: &mut Commands,
    left: f32,
    top: f32,
    width: f32,
    label: &str,
    action: UiCommandButton,
) {
    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(width),
                height: Val::Px(27.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.075, 0.105, 0.135, 0.96)),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.87, 0.91)),
            ));
        });
}

fn spawn_country_flag(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(282.0),
            top: Val::Px(112.0),
            width: Val::Px(46.0),
            height: Val::Px(32.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.18, 0.46, 0.43)),
        CountryFlag,
    ));
}

fn spawn_metric_bar(
    commands: &mut Commands,
    top: f32,
    label: &str,
    metric: MetricBar,
    color: Color,
) {
    commands.spawn((
        Text::new(label),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::srgb(0.59, 0.68, 0.75)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(220.0),
            top: Val::Px(top - 14.0),
            width: Val::Px(105.0),
            ..default()
        },
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(28.0),
            top: Val::Px(top),
            width: Val::Px(292.0),
            height: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.88)),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(28.0),
            top: Val::Px(top),
            width: Val::Px(146.0),
            height: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(color),
        metric,
    ));
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn handle_ui_buttons(
    interactions: Query<(&Interaction, &UiCommandButton), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<InspectorState>,
    mut flow: ResMut<TimeFlow>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            UiCommandButton::Map(mode) => state.overlay = *mode,
            UiCommandButton::Pause => flow.paused = !flow.paused,
            UiCommandButton::Speed(speed) => {
                flow.speed = *speed;
                flow.paused = false;
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn refresh_ui_buttons(
    state: Res<InspectorState>,
    flow: Res<TimeFlow>,
    mut buttons: Query<(&Interaction, &UiCommandButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, action, mut background) in &mut buttons {
        let active = match action {
            UiCommandButton::Map(mode) => state.overlay == *mode,
            UiCommandButton::Pause => flow.paused,
            UiCommandButton::Speed(speed) => !flow.paused && flow.speed == *speed,
        };
        background.0 = if *interaction == Interaction::Pressed || active {
            Color::srgb(0.68, 0.47, 0.16)
        } else if *interaction == Interaction::Hovered {
            Color::srgb(0.18, 0.30, 0.36)
        } else {
            Color::srgb(0.075, 0.105, 0.135)
        };
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn handle_observatory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut simulation: ResMut<Simulation>,
    mut observatory: ResMut<Observatory>,
    mut state: ResMut<InspectorState>,
    mut desk: ResMut<ProgramDeskState>,
    mut flow: ResMut<TimeFlow>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        state.overlay = OverlayMode::Political;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        state.overlay = OverlayMode::Confidence;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        state.overlay = OverlayMode::Population;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        state.overlay = OverlayMode::Output;
    }
    if keyboard.just_pressed(KeyCode::Tab) && !observatory.0.regions.is_empty() {
        state.selected = (state.selected + 1) % observatory.0.regions.len();
    }
    if keyboard.just_pressed(KeyCode::Space) {
        flow.paused = !flow.paused;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        flow.speed = match flow.speed {
            0..=1 => 4,
            2..=4 => 12,
            5..=12 => 24,
            _ => 72,
        };
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        flow.speed = match flow.speed {
            0..=4 => 1,
            5..=12 => 4,
            13..=24 => 12,
            _ => 24,
        };
    }
    if keyboard.just_pressed(KeyCode::KeyH) {
        match WorldCommand::AdvanceHour.apply(&mut simulation.0) {
            Ok(()) => {
                observatory.0 = ObservatorySnapshot::capture(&simulation.0);
                "Проведён один час симуляции.".clone_into(&mut desk.message);
            }
            Err(_) => {
                "Переход часа отклонён правилами симуляции.".clone_into(&mut desk.message);
            }
        }
    }
    let actor = adam_content::OBSERVATORY_ACTOR_ID;
    let program = adam_content::OBSERVATORY_PROGRAM_ID;
    let promised = simulation.0.government_programs().get(&program).map_or(
        Money::default(),
        adam_core::GovernmentProgram::promised_annual_funding,
    );
    let amount = Money::from_minor_units(promised.minor_units() / 2);
    let command = if keyboard.just_pressed(KeyCode::KeyT) {
        Some((
            "Финансирование из казны",
            WorldCommand::AppropriateGovernmentProgram {
                actor,
                program,
                amount,
                source: ProgramFundingSource::Treasury,
            },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyD) {
        Some((
            "Долговое финансирование",
            WorldCommand::AppropriateGovernmentProgram {
                actor,
                program,
                amount,
                source: ProgramFundingSource::PublicDebt,
            },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyE) {
        Some((
            "Исполнение программы",
            WorldCommand::ExecuteGovernmentProgram { actor, program },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyC) {
        Some((
            "Отмена программы",
            WorldCommand::CancelGovernmentProgram { actor, program },
        ))
    } else {
        None
    };
    if let Some((label, command)) = command {
        match command.apply(&mut simulation.0) {
            Ok(()) => {
                observatory.0 = ObservatorySnapshot::capture(&simulation.0);
                desk.message = format!("{label}: решение принято.");
            }
            Err(_) => {
                desk.message = format!("{label}: решение отклонено правилами симуляции.");
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn update_map_pointer(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Query<&Transform, With<MapCamera>>,
    observatory: Res<Observatory>,
    geometry: Res<MapGeometry>,
    mut hover: ResMut<MapHover>,
    mut state: ResMut<InspectorState>,
    mut tooltip: Query<(&mut Text, &mut Node, &mut Visibility), With<MapTooltipText>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        hover.cell = None;
        hover.region = None;
        if let Ok((_, _, mut visibility)) = tooltip.single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };
    let scale = camera.scale.x;
    let world = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale + camera.translation.x,
        (window.height() * 0.5 - cursor.y) * scale + camera.translation.y,
    );
    let blocked_by_ui = cursor.y < 90.0
        || cursor.x > window.width() - 420.0
        || cursor.y > window.height() - 250.0
        || (cursor.x < 250.0 && cursor.y < 400.0);
    let found = if blocked_by_ui {
        None
    } else {
        geometry.0.cell_at(world)
    };
    let found_region = found.and_then(|index| geometry.0.cells[index].region());
    hover.cell = found;
    hover.region = found_region;
    hover.screen_position = cursor;
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(region) = found_region {
            state.selected = region;
        }
    }
    if let Ok((mut text, mut node, mut visibility)) = tooltip.single_mut() {
        if let Some((cell, region_index)) = found.zip(found_region) {
            let Some(region) = observatory.0.regions.get(region_index) else {
                *visibility = Visibility::Hidden;
                return;
            };
            text.0 = format!(
                "{}  /  {}\nПровинция №{}   {}\nНаселение: {}\nВыпуск: {}\nДоверие: {}.{:02}%",
                localized_region_name(&region.name).to_uppercase(),
                localized_country_name(region.country.get()),
                cell,
                localized_relief(geometry.0.cells[cell].relief),
                region.population,
                region.annual_output.minor_units(),
                region.satisfaction.get() / 100,
                region.satisfaction.get() % 100,
            );
            node.left = Val::Px((cursor.x + 18.0).min(window.width() - 260.0));
            node.top = Val::Px((cursor.y + 18.0).min(window.height() - 140.0));
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn control_map_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: EventReader<MouseWheel>,
    mut motion: EventReader<MouseMotion>,
    geometry: Res<MapGeometry>,
    mut camera: Query<&mut Transform, With<MapCamera>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        wheel.clear();
        motion.clear();
        return;
    };
    let mut zoom = 1.0_f32;
    for event in wheel.read() {
        zoom *= 1.0 - event.y.clamp(-3.0, 3.0) * 0.12;
    }
    let mut drag = Vec2::ZERO;
    let dragging = mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle);
    for event in motion.read() {
        if dragging {
            drag += Vec2::new(-event.delta.x, event.delta.y);
        }
    }
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyI) || keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyK) || keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyJ) || keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyL) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyU) {
        zoom *= 1.0 - time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyO) {
        zoom *= 1.0 + time.delta_secs();
    }
    let scale = (transform.scale.x * zoom).clamp(0.42, 2.10);
    transform.scale = Vec3::splat(scale);
    let mut translation = transform.translation.truncate() + drag * scale;
    if direction != Vec2::ZERO {
        translation += direction.normalize() * 620.0 * time.delta_secs() * scale;
    }
    let limit = geometry.0.half_extent;
    transform.translation.x = translation.x.clamp(-limit.x, limit.x);
    transform.translation.y = translation.y.clamp(-limit.y, limit.y);
    if keyboard.just_pressed(KeyCode::KeyR) {
        transform.translation = Vec3::ZERO;
        transform.scale = Vec3::splat(1.35);
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn update_map_lod(
    camera: Query<&Transform, With<MapCamera>>,
    mut elements: Query<
        (
            &mut Visibility,
            Option<&RegionLabel>,
            Option<&CountryLabel>,
            Option<&ReliefMarker>,
            Option<&CapitalMarker>,
        ),
        Or<(
            With<RegionLabel>,
            With<CountryLabel>,
            With<ReliefMarker>,
            With<CapitalMarker>,
        )>,
    >,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let scale = camera.scale.x;
    for (mut visibility, region, country, relief, capital) in &mut elements {
        let visible = if region.is_some() {
            scale <= 1.55
        } else if country.is_some() {
            scale >= 0.90
        } else if relief.is_some() {
            scale <= 1.20
        } else if capital.is_some() {
            scale <= 1.80
        } else {
            true
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
fn advance_game_clock(
    time: Res<Time>,
    mut flow: ResMut<TimeFlow>,
    mut simulation: ResMut<Simulation>,
    mut observatory: ResMut<Observatory>,
    mut desk: ResMut<ProgramDeskState>,
) {
    if flow.paused {
        return;
    }
    flow.accumulated_hours += time.delta_secs() * f32::from(flow.speed);
    let mut hours = 0_u8;
    while hours < 24 && flow.accumulated_hours >= 1.0 {
        flow.accumulated_hours -= 1.0;
        hours += 1;
    }
    if hours == 0 {
        return;
    }
    let mut monthly_tick = false;
    for _ in 0..hours {
        if let Ok(ticked) = simulation.0.advance_hour() {
            monthly_tick |= ticked;
        } else {
            flow.paused = true;
            "Время остановлено из-за ошибки симуляции.".clone_into(&mut desk.message);
            return;
        }
    }
    observatory.0 = ObservatorySnapshot::capture(&simulation.0);
    if monthly_tick {
        "На границе месяца выполнен экономический расчёт.".clone_into(&mut desk.message);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn refresh_header(
    observatory: Res<Observatory>,
    flow: Res<TimeFlow>,
    mut text: Query<&mut Text, With<DateText>>,
) {
    if !observatory.is_changed() && !flow.is_changed() {
        return;
    }
    if let Ok(mut text) = text.single_mut() {
        text.0 = header_text(&observatory.0, flow.paused, flow.speed);
    }
}

fn header_text(snapshot: &ObservatorySnapshot, paused: bool, speed: u8) -> String {
    let state = if paused {
        "ПАУЗА".to_owned()
    } else {
        format!("ХОД ВРЕМЕНИ x{speed}")
    };
    format!(
        "{:02}.{:02}.{}  {:02}:00    {}    РЕГИОНОВ: {}",
        snapshot.date_day,
        snapshot.date_month,
        snapshot.date_year,
        snapshot.date_hour,
        state,
        snapshot.regions.len()
    )
}

#[allow(clippy::needless_pass_by_value)]
fn refresh_map_colors(
    observatory: Res<Observatory>,
    state: Res<InspectorState>,
    hover: Res<MapHover>,
    geometry: Res<MapGeometry>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cells: Query<(&ProvinceCell, &MeshMaterial2d<ColorMaterial>)>,
    mut borders: Query<(&BorderLine, &mut Sprite)>,
) {
    if !state.is_changed() && !observatory.is_changed() && !hover.is_changed() {
        return;
    }
    for (province, handle) in &cells {
        let Some(cell) = geometry.0.cells.get(province.cell) else {
            continue;
        };
        let mut color = cell_color(cell, &observatory.0, state.overlay);
        if hover.cell == Some(province.cell) {
            color = color.mix(&Color::WHITE, 0.16);
        } else if hover.region == Some(province.region) {
            color = color.mix(&Color::WHITE, 0.06);
        }
        if province.region == state.selected {
            color = color.mix(&Color::WHITE, 0.10);
        }
        if let Some(material) = materials.get_mut(&handle.0) {
            material.color = color;
        }
    }
    for (line, mut sprite) in &mut borders {
        let touches_selection =
            line.left == Some(state.selected) || line.right == Some(state.selected);
        let touches_hover = hover
            .region
            .is_some_and(|region| line.left == Some(region) || line.right == Some(region));
        let outer = line.class != BorderClass::Province;
        sprite.color = if touches_selection && outer {
            Color::srgb(0.96, 0.78, 0.30)
        } else if touches_hover && outer {
            Color::srgb(0.46, 0.80, 0.90)
        } else {
            border_style(line.class).1
        };
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn refresh_inspector_panels(
    observatory: Res<Observatory>,
    state: Res<InspectorState>,
    mut inspector: Query<
        &mut Text,
        (
            With<InspectorText>,
            Without<OverlayText>,
            Without<LegendText>,
        ),
    >,
    mut overlay: Query<
        &mut Text,
        (
            With<OverlayText>,
            Without<InspectorText>,
            Without<LegendText>,
        ),
    >,
    mut legend: Query<
        &mut Text,
        (
            With<LegendText>,
            Without<OverlayText>,
            Without<InspectorText>,
        ),
    >,
) {
    if !state.is_changed() && !observatory.is_changed() {
        return;
    }
    if let Ok(mut text) = inspector.single_mut() {
        text.0 = inspector_text(observatory.0.regions.get(state.selected));
    }
    if let Ok(mut text) = overlay.single_mut() {
        text.0 = format!("СЛОИ КАРТЫ  /  {}", state.overlay.label().to_uppercase());
    }
    if let Ok(mut text) = legend.single_mut() {
        text.0 = legend_text(state.overlay);
    }
}

fn legend_text(overlay: OverlayMode) -> String {
    format!(
        "КАМЕРА\nПКМ — перетаскивание   Колесо — масштаб\nWASD/IJKL — движение   R — сброс\nЛКМ по провинции — выбор региона\n\n{}",
        match overlay {
            OverlayMode::Political => "Аркадия — бирюзовая   Бореалия — красная",
            OverlayMode::Confidence => "Красный — кризис   Зелёный — доверие",
            OverlayMode::Population => "Тёмный — малонаселённый   Яркий — плотный",
            OverlayMode::Output => "Тёмный — слабый выпуск   Яркий — высокий",
        }
    )
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn refresh_metric_bars(
    observatory: Res<Observatory>,
    state: Res<InspectorState>,
    mut bars: Query<(&MetricBar, &mut Node), Without<CountryFlag>>,
    mut flag: Query<&mut BackgroundColor, (With<CountryFlag>, Without<MetricBar>)>,
) {
    if !observatory.is_changed() && !state.is_changed() {
        return;
    }
    let Some(region) = observatory.0.regions.get(state.selected) else {
        return;
    };
    let max_population = observatory
        .0
        .regions
        .iter()
        .map(|region| region.population)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_output = observatory
        .0
        .regions
        .iter()
        .map(|region| region.annual_output.minor_units().max(0))
        .max()
        .unwrap_or(1)
        .max(1);
    for (metric, mut node) in &mut bars {
        let ratio = match metric {
            MetricBar::Confidence => f32::from(region.satisfaction.get()) / 10_000.0,
            MetricBar::Population => region.population as f32 / max_population as f32,
            MetricBar::Output => {
                region.annual_output.minor_units().max(0) as f32 / max_output as f32
            }
        };
        node.width = Val::Px(292.0 * ratio.clamp(0.0, 1.0));
    }
    if let Ok(mut flag) = flag.single_mut() {
        flag.0 = country_color(region.country.get());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn refresh_national_status(
    observatory: Res<Observatory>,
    state: Res<InspectorState>,
    mut text: Query<&mut Text, With<NationalStatusText>>,
) {
    if !observatory.is_changed() && !state.is_changed() {
        return;
    }
    if let Ok(mut text) = text.single_mut() {
        text.0 = national_status_text(&observatory.0, state.selected);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn refresh_program_desk(
    observatory: Res<Observatory>,
    desk: Res<ProgramDeskState>,
    mut text: Query<&mut Text, With<ProgramDeskText>>,
) {
    if !observatory.is_changed() && !desk.is_changed() {
        return;
    }
    let Some(program) = observatory.0.programs.first() else {
        return;
    };
    if let Ok(mut text) = text.single_mut() {
        text.0 = format!(
            "ГОСУДАРСТВЕННАЯ ПРОГРАММА\n\n{}  [{}]\nОбещано: {}\nАссигновано: {}\nИсполнено: {}\nПеренос: {}\nЛет задержки: {}\n\n[T] казна  [D] госдолг  [E] исполнить\n[C] отменить\n\nВремя: [Space] пауза  [↑/↓] скорость  [H] один час\n\n{}",
            localized_program_name(&program.name),
            localized_program_status(program.status),
            program.promised.minor_units(),
            program.appropriated.minor_units(),
            program.delivered.minor_units(),
            program.carryover.minor_units(),
            program.years_delayed,
            desk.message,
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn refresh_timeline(observatory: Res<Observatory>, mut text: Query<&mut Text, With<TimelineText>>) {
    if !observatory.is_changed() {
        return;
    }
    if let Ok(mut text) = text.single_mut() {
        text.0 = timeline_text(&observatory.0);
    }
}

fn timeline_text(snapshot: &ObservatorySnapshot) -> String {
    let mut lines = vec!["ПОЛИТИЧЕСКАЯ ЛЕТОПИСЬ".to_owned()];
    if snapshot.chronicle.is_empty() {
        lines.push("Политические события пока не зафиксированы.".to_owned());
    } else {
        for entry in snapshot.chronicle.iter().rev().take(4).rev() {
            lines.push(format!(
                "{}  | важность {} |  {}",
                entry.year,
                entry.importance,
                localized_chronicle_text(&entry.text)
            ));
        }
    }
    lines.join("\n\n")
}

fn inspector_text(region: Option<&RegionSnapshot>) -> String {
    region.map_or_else(
        || "Регион не выбран".to_owned(),
        |region| {
            format!(
                "ИНСПЕКТОР РЕГИОНА\n\n{}\nРегион {} / {}\n\nНаселение: {}\nГодовой выпуск: {}\nДоверие: {}.{:02}%",
                localized_region_name(&region.name),
                region.id.get(),
                localized_country_name(region.country.get()),
                region.population,
                region.annual_output.minor_units(),
                region.satisfaction.get() / 100,
                region.satisfaction.get() % 100
            )
        },
    )
}

fn national_status_text(snapshot: &ObservatorySnapshot, selected_region: usize) -> String {
    let country_id = snapshot
        .regions
        .get(selected_region)
        .map(|region| region.country)
        .or_else(|| snapshot.countries.first().map(|country| country.id));
    let Some(country_id) = country_id else {
        return "Государственные показатели недоступны".to_owned();
    };
    let Some(country) = snapshot
        .countries
        .iter()
        .find(|country| country.id == country_id)
    else {
        return "Государственные показатели недоступны".to_owned();
    };
    format!(
        "{}   КАЗНА {}   ДОЛГ {}   ЛЕГИТИМНОСТЬ {}%   СПЛОЧЁННОСТЬ ЭЛИТ {}%",
        localized_country_name(country.id.get()).to_uppercase(),
        country.treasury.minor_units(),
        country.public_debt.minor_units(),
        country.legitimacy.get() / 100,
        country.elite_cohesion.get() / 100,
    )
}

fn localized_country_name(id: u32) -> &'static str {
    match id {
        1 => "Аркадия",
        2 => "Бореалия",
        _ => "Неизвестная страна",
    }
}

fn localized_region_name(name: &str) -> &str {
    match name {
        "Northreach" => "Северный край",
        "Southvale" => "Южная долина",
        "Eastport" => "Восточный порт",
        _ => name,
    }
}

const fn localized_relief(relief: Relief) -> &'static str {
    match relief {
        Relief::Plain => "равнина",
        Relief::Forest => "леса",
        Relief::Hills => "холмы",
        Relief::Mountains => "горы",
    }
}

fn localized_program_name(name: &str) -> &str {
    match name {
        "National Renewal Program" => "Национальная программа обновления",
        _ => name,
    }
}

const fn localized_program_status(status: adam_core::GovernmentProgramStatus) -> &'static str {
    match status {
        adam_core::GovernmentProgramStatus::Announced => "ОБЪЯВЛЕНА",
        adam_core::GovernmentProgramStatus::Active => "ДЕЙСТВУЕТ",
        adam_core::GovernmentProgramStatus::Completed => "ЗАВЕРШЕНА",
        adam_core::GovernmentProgramStatus::Cancelled => "ОТМЕНЕНА",
        adam_core::GovernmentProgramStatus::Failed => "ПРОВАЛЕНА",
    }
}

fn localized_chronicle_text(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if lower.contains("cancel") {
        "Государственная программа отменена политическим решением."
    } else if lower.contains("declar") || lower.contains("announc") {
        "Объявлена новая государственная программа и зафиксировано публичное обещание."
    } else if lower.contains("appropriat") || lower.contains("fund") || lower.contains("debt") {
        "Принято решение о финансировании государственной программы."
    } else if lower.contains("deliver") || lower.contains("execut") || lower.contains("carryover") {
        "Зафиксированы исполнение программы, поставки и неисполненный перенос."
    } else if lower.contains("legitim") || lower.contains("polar") || lower.contains("cohesion") {
        "Последствия программы изменили легитимность, поляризацию и сплочённость элит."
    } else if lower.contains("migration") || lower.contains("housing") {
        "Зафиксированы миграционные и жилищные изменения в регионах."
    } else if lower.contains("shortage") || lower.contains("relief") {
        "Материальный дефицит вызвал государственное вмешательство и общественные последствия."
    } else {
        "Зафиксировано значимое событие государственной летописи."
    }
}

fn country_color(id: u32) -> Color {
    match id {
        1 => Color::srgb(0.17, 0.44, 0.42),
        2 => Color::srgb(0.52, 0.25, 0.24),
        other => {
            let tone = (other % 5) as f32 * 0.06;
            Color::srgb(0.24 + tone, 0.34, 0.46 - tone * 0.5)
        }
    }
}

fn cell_color(cell: &MapCell, snapshot: &ObservatorySnapshot, overlay: OverlayMode) -> Color {
    let Some(region_index) = cell.region() else {
        return shift_lightness(OCEAN_SHELF, (cell.tint - 0.5) * 0.035);
    };
    let Some(region) = snapshot.regions.get(region_index) else {
        return shift_lightness(OCEAN_SHELF, (cell.tint - 0.5) * 0.035);
    };
    let base = if overlay == OverlayMode::Political {
        shift_lightness(
            country_color(region.country.get()),
            ((region_index % 3) as f32 - 1.0) * 0.035,
        )
    } else {
        data_color(overlay_ratio(region, overlay, &snapshot.regions))
    };
    let relief_shade = match cell.relief {
        Relief::Mountains => 0.035,
        Relief::Hills => 0.015,
        Relief::Forest => -0.030,
        Relief::Plain => 0.0,
    };
    shift_lightness(base, (cell.tint - 0.5) * 0.055 + relief_shade)
}

fn overlay_ratio(region: &RegionSnapshot, overlay: OverlayMode, regions: &[RegionSnapshot]) -> f32 {
    match overlay {
        OverlayMode::Political => 0.0,
        OverlayMode::Confidence => f32::from(region.satisfaction.get()) / 10_000.0,
        OverlayMode::Population => {
            let maximum = regions
                .iter()
                .map(|candidate| candidate.population)
                .max()
                .unwrap_or(1)
                .max(1);
            region.population as f32 / maximum as f32
        }
        OverlayMode::Output => {
            let maximum = regions
                .iter()
                .map(|candidate| candidate.annual_output.minor_units().max(0))
                .max()
                .unwrap_or(1)
                .max(1);
            region.annual_output.minor_units().max(0) as f32 / maximum as f32
        }
    }
}

fn data_color(ratio: f32) -> Color {
    let bounded = ratio.clamp(0.0, 1.0);
    let (low, high, local) = if bounded < 0.5 {
        (
            Color::srgb(0.50, 0.15, 0.17),
            Color::srgb(0.80, 0.64, 0.24),
            bounded * 2.0,
        )
    } else {
        (
            Color::srgb(0.80, 0.64, 0.24),
            Color::srgb(0.20, 0.56, 0.34),
            (bounded - 0.5) * 2.0,
        )
    };
    low.mix(&high, local)
}

fn shift_lightness(color: Color, amount: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgb(
        (srgba.red + amount).clamp(0.0, 1.0),
        (srgba.green + amount).clamp(0.0, 1.0),
        (srgba.blue + amount).clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod visual_tests {
    use super::*;

    fn sample_geometry() -> (ObservatorySnapshot, StrategicMap) {
        let world = adam_content::observatory_world(5).expect("world");
        let snapshot = ObservatorySnapshot::capture(&world);
        let ownership: Vec<u32> = snapshot
            .regions
            .iter()
            .map(|region| region.country.get())
            .collect();
        let map = StrategicMap::generate(&ownership, MAP_SEED);
        (snapshot, map)
    }

    #[test]
    fn every_authoritative_region_owns_provinces_on_the_map() {
        let (snapshot, map) = sample_geometry();
        assert_eq!(map.region_cell_counts.len(), snapshot.regions.len());
        assert!(map.region_cell_counts.iter().all(|count| *count > 0));
    }

    #[test]
    fn province_hit_testing_resolves_to_the_owning_region() {
        let (_, map) = sample_geometry();
        let province = map
            .cells
            .iter()
            .position(|cell| cell.region().is_some())
            .expect("land province");
        let center = map.cells[province].center;
        assert_eq!(map.cell_at(center), Some(province));
        assert_eq!(map.region_at(center), map.cells[province].region());
        let outside = map.half_extent * 4.0;
        assert_eq!(map.cell_at(outside), None);
    }

    #[test]
    fn political_colouring_separates_countries() {
        let (snapshot, map) = sample_geometry();
        let mut arcadia = None;
        let mut borealia = None;
        for cell in &map.cells {
            let Some(region) = cell.region().and_then(|index| snapshot.regions.get(index)) else {
                continue;
            };
            let color = cell_color(cell, &snapshot, OverlayMode::Political).to_srgba();
            match region.country.get() {
                1 => arcadia = Some(color),
                2 => borealia = Some(color),
                _ => {}
            }
        }
        let arcadia = arcadia.expect("arcadian province");
        let borealia = borealia.expect("borealian province");
        assert!(arcadia.red < borealia.red);
        assert!(arcadia.green > borealia.green);
    }

    #[test]
    fn data_overlay_scales_from_crisis_to_confidence() {
        assert!(data_color(0.0).to_srgba().red > data_color(1.0).to_srgba().red);
        assert!(data_color(1.0).to_srgba().green > data_color(0.0).to_srgba().green);
    }

    #[test]
    fn visible_authoritative_names_have_russian_presentation() {
        assert_eq!(localized_country_name(1), "Аркадия");
        assert_eq!(localized_country_name(2), "Бореалия");
        assert_eq!(localized_region_name("Northreach"), "Северный край");
        assert_eq!(
            localized_program_name("National Renewal Program"),
            "Национальная программа обновления"
        );
        assert_eq!(
            localized_program_status(adam_core::GovernmentProgramStatus::Active),
            "ДЕЙСТВУЕТ"
        );
    }

    #[test]
    fn interface_font_is_embedded_and_covers_cyrillic() {
        assert!(INTERFACE_FONT_BYTES.len() > 100_000);
        Font::try_from_bytes(INTERFACE_FONT_BYTES.to_vec()).expect("embedded font parses");
    }

    #[test]
    fn strategic_header_uses_authoritative_country_metrics_in_russian() {
        let world = adam_content::observatory_world(31).expect("world");
        let snapshot = ObservatorySnapshot::capture(&world);
        let text = national_status_text(&snapshot, 0);
        assert!(text.contains("АРКАДИЯ"));
        assert!(text.contains("КАЗНА"));
        assert!(text.contains("ЛЕГИТИМНОСТЬ"));
        assert!(!text.contains("Treasury"));
    }

    #[test]
    fn graphical_chronicle_has_no_english_fallback() {
        let world = adam_content::observatory_world(37).expect("world");
        let text = timeline_text(&ObservatorySnapshot::capture(&world));
        assert!(text.contains("ПОЛИТИЧЕСКАЯ ЛЕТОПИСЬ"));
        assert!(!text.contains("POLITICAL TIMELINE"));
        assert!(!text.contains("importance"));
    }
}
