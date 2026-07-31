use adam_core::{Money, ProgramFundingSource, World, WorldCommand};
use adam_observatory::{ObservatorySnapshot, RegionSnapshot};
use bevy::prelude::*;
use bevy::window::WindowResolution;

#[derive(Resource)]
struct Observatory(ObservatorySnapshot);

#[derive(Resource)]
struct Simulation(World);

#[derive(Resource)]
struct ProgramDeskState {
    message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum OverlayMode {
    Confidence,
    Population,
    Output,
}

impl OverlayMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Confidence => "Confidence",
            Self::Population => "Population",
            Self::Output => "Annual output",
        }
    }
}

#[derive(Resource)]
struct InspectorState {
    selected: usize,
    overlay: OverlayMode,
}

#[derive(Component)]
struct RegionCard(usize);

#[derive(Component)]
struct InspectorText;

#[derive(Component)]
struct OverlayText;

#[derive(Component)]
struct ProgramDeskText;

#[derive(Component)]
struct TimelineText;

fn main() {
    let world = adam_content::observatory_world(1).expect("observatory world must load");
    let snapshot = ObservatorySnapshot::capture(&world);
    App::new()
        .insert_resource(Observatory(snapshot))
        .insert_resource(Simulation(world))
        .insert_resource(ProgramDeskState {
            message: "Program announced. Choose a funding source.".to_owned(),
        })
        .insert_resource(InspectorState {
            selected: 0,
            overlay: OverlayMode::Confidence,
        })
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "A.D.A.M — Governing State Observatory".to_owned(),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_observatory)
        .add_systems(
            Update,
            (
                handle_observatory_input,
                refresh_observatory,
                refresh_program_desk,
                refresh_timeline,
            )
                .chain(),
        )
        .run();
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn setup_observatory(mut commands: Commands, observatory: Res<Observatory>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new(format!(
            "A.D.A.M  |  {} day {}  |  {} regions",
            observatory.0.date_year,
            observatory.0.date_day,
            observatory.0.regions.len()
        )),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.88, 0.92, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(18.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new(
            "Overlay: Confidence  |  [1] confidence  [2] population  [3] output  [Tab] inspect",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.63, 0.72, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(54.0),
            ..default()
        },
        OverlayText,
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
            width: Val::Px(300.0),
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
        Text::new("PROGRAM DESK"),
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
    for (index, region) in observatory.0.regions.iter().enumerate() {
        commands.spawn((
            Sprite::from_color(
                region_color(region, OverlayMode::Confidence, &observatory.0.regions),
                Vec2::new(190.0, 125.0),
            ),
            Transform::from_xyz(region.map_x as f32, region.map_y as f32, 0.0),
            RegionCard(index),
        ));
        commands.spawn((
            Text2d::new(format!(
                "{}\nPop {}\nOutput {}\nConfidence {}.{:02}%",
                region.name,
                region.population,
                region.annual_output.minor_units(),
                region.satisfaction.get() / 100,
                region.satisfaction.get() % 100
            )),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_xyz(region.map_x as f32, region.map_y as f32, 1.0),
        ));
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn handle_observatory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut simulation: ResMut<Simulation>,
    mut observatory: ResMut<Observatory>,
    mut state: ResMut<InspectorState>,
    mut desk: ResMut<ProgramDeskState>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        state.overlay = OverlayMode::Confidence;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        state.overlay = OverlayMode::Population;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        state.overlay = OverlayMode::Output;
    }
    if keyboard.just_pressed(KeyCode::Tab) && !observatory.0.regions.is_empty() {
        state.selected = (state.selected + 1) % observatory.0.regions.len();
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
            "Treasury appropriation",
            WorldCommand::AppropriateGovernmentProgram {
                actor,
                program,
                amount,
                source: ProgramFundingSource::Treasury,
            },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyD) {
        Some((
            "Debt appropriation",
            WorldCommand::AppropriateGovernmentProgram {
                actor,
                program,
                amount,
                source: ProgramFundingSource::PublicDebt,
            },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyE) {
        Some((
            "Program execution",
            WorldCommand::ExecuteGovernmentProgram { actor, program },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyC) {
        Some((
            "Program cancellation",
            WorldCommand::CancelGovernmentProgram { actor, program },
        ))
    } else if keyboard.just_pressed(KeyCode::KeyY) {
        Some(("Annual advance", WorldCommand::AdvanceEconomicYear))
    } else {
        None
    };
    if let Some((label, command)) = command {
        match command.apply(&mut simulation.0) {
            Ok(()) => {
                observatory.0 = ObservatorySnapshot::capture(&simulation.0);
                desk.message = format!("{label} accepted by adam-core.");
            }
            Err(error) => desk.message = format!("{label} rejected: {error:?}"),
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn refresh_observatory(
    observatory: Res<Observatory>,
    state: Res<InspectorState>,
    mut cards: Query<(&RegionCard, &mut Sprite)>,
    mut inspector: Query<&mut Text, (With<InspectorText>, Without<OverlayText>)>,
    mut overlay: Query<&mut Text, (With<OverlayText>, Without<InspectorText>)>,
) {
    if !state.is_changed() {
        return;
    }
    for (card, mut sprite) in &mut cards {
        let region = &observatory.0.regions[card.0];
        let mut color = region_color(region, state.overlay, &observatory.0.regions);
        if card.0 == state.selected {
            color = color.mix(&Color::WHITE, 0.20);
        }
        sprite.color = color;
    }
    if let Ok(mut text) = inspector.single_mut() {
        text.0 = inspector_text(observatory.0.regions.get(state.selected));
    }
    if let Ok(mut text) = overlay.single_mut() {
        text.0 = format!(
            "Overlay: {}  |  [1] confidence  [2] population  [3] output  [Tab] inspect",
            state.overlay.label()
        );
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
            "PROGRAM DESK\n\n{}  [{:?}]\nPromised: {}\nAppropriated: {}\nDelivered: {}\nCarryover: {}\nDelayed years: {}\n\n[T] treasury  [D] debt  [E] execute\n[C] cancel  [Y] advance year\n\n{}",
            program.name,
            program.status,
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
    let mut lines = vec!["POLITICAL TIMELINE".to_owned()];
    if snapshot.chronicle.is_empty() {
        lines.push("No political history recorded yet.".to_owned());
    } else {
        for entry in snapshot.chronicle.iter().rev().take(4).rev() {
            lines.push(format!(
                "{}  ◆{}  {}",
                entry.year, entry.importance, entry.text
            ));
        }
    }
    lines.join("\n\n")
}

fn inspector_text(region: Option<&RegionSnapshot>) -> String {
    region.map_or_else(
        || "No region selected".to_owned(),
        |region| {
            format!(
                "REGION INSPECTOR\n\n{}\nRegion {} / Country {}\n\nPopulation: {}\nAnnual output: {}\nConfidence: {}.{:02}%",
                region.name,
                region.id.get(),
                region.country.get(),
                region.population,
                region.annual_output.minor_units(),
                region.satisfaction.get() / 100,
                region.satisfaction.get() % 100
            )
        },
    )
}

fn region_color(
    region: &RegionSnapshot,
    overlay: OverlayMode,
    regions: &[RegionSnapshot],
) -> Color {
    let ratio = match overlay {
        OverlayMode::Confidence => f32::from(region.satisfaction.get()) / 10_000.0,
        OverlayMode::Population => {
            let maximum = regions
                .iter()
                .map(|candidate| candidate.population)
                .max()
                .unwrap_or(1);
            region.population as f32 / maximum.max(1) as f32
        }
        OverlayMode::Output => {
            let maximum = regions
                .iter()
                .map(|candidate| candidate.annual_output.minor_units().max(0))
                .max()
                .unwrap_or(1);
            region.annual_output.minor_units().max(0) as f32 / maximum.max(1) as f32
        }
    };
    let bounded = ratio.clamp(0.0, 1.0);
    Color::srgb(
        0.12 + 0.34 * (1.0 - bounded),
        0.17 + 0.58 * bounded,
        0.30 + 0.18 * bounded,
    )
}
