//! On mouse click,
//! - check all Draggable entities,
//! - check if they are under mouse,
//! - add BeingDragged to them
//!
//! On mouse move,
//! - translate all BeingDragged entities by mouse delta
//! - TODO: check boundaries
//!
//! On mouse release,
//! - remove BeingDragged from all
//!
//! TODO list
//! - [x] Create dossier
//!     - Add text for dossier
//!     - Add image
//! - [ ] Create newspaper
//! - [x] Create stamper
//! - [ ] Create game logic
//! - [x] Create checklist for your job
//!
//!
//! Game flow:
//! 1. Title screen:
//!         Welcome to the CIA
//!             Start Game
//! 2. Some text "Welcome to your first day as an analyst at the CIA!"
//! 3. Drag dossiers from inbox onto desk (don't all have to be people. could be other issues)
//! 4. Choose stamp to place under "Analyst Recommendation" (for foreign leaders, assassinate or respect)
//! 5. Drag dossier into outbox
//! 6. Day ends
//!
//! MVP Game flow
//! 1. "Welcome to your first day as an analyst at the CIA!"
//! 2. (Click continue)
//! 3. Two options for Nasser dossier: assassinate or respect
//! 4. Click continue
//! 5. If you clicked assassinate, show real newspaper of egypt mourning him
//!    If you clicked respect, show letter of termination explaining why you're fired
//! Ideas
//! - Ga
//!
//! You either put a stamp of "Coup", "Assassinate", or "Ignore"
//!
//! Screen fades to black when the day ends
//! Newspaper arrives on your desk
//! - If you said "Coup"
//!   - Egyptian military storms capital, killing president
//! - If you said "Assassinate"
//!   - "Beloved president dies in his sleep of a heart attack"
//!      - Picture of his child crying
//! - If you said "Ignore"
//!   - You get a note saying you're fired
//!
//!
#![allow(clippy::type_complexity)]
use std::mem;

use bevy::{prelude::*, text::Text2dBounds};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};
use ordered_float::NotNan;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_loopless_state(GameState::Desk)
        .add_startup_system(setup)
        .add_enter_system(GameState::Desk, spawn_dossier)
        .add_enter_system(GameState::Desk, spawn_checklist)
        .add_enter_system(GameState::Desk, spawn_stamp)
        .add_system(begin_being_dragged)
        .add_system(stop_being_dragged)
        .add_system(drag)
        .add_system(check_timer.run_in_state(GameState::Desk))
        .add_exit_system(GameState::Desk, despawn_desk)
        .add_enter_system(GameState::Newspaper, spawn_newspaper)
        .add_system(calc_mouse_pos)
        .run();
}

fn despawn_desk(query: Query<Entity, With<OnDesk>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_newspaper(mut commands: Commands, asset_server: Res<AssetServer>, windows: Res<Windows>) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("NYTimes-Edited.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                windows.primary().width(),
                windows.primary().height(),
            )),
            ..default()
        },
        // transform: Transform::from_xyz(
        //     // Horizontally centered on the page
        //     paper_position.x,
        //     paper_position.y + paper_size.y / 2.0 - logo_size.y / 2.0,
        //     1.0,
        // ),
        ..default()
    });
}

#[derive(Component)]
struct OnDesk;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn spawn_checklist(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paper_size = Vec2::new(300.0, 300.0 * 11.0 / 8.5);
    let paper_position = Vec2::new(250.0, 100.0);
    let logo_size = Vec2::new(40.0, 40.0);
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let instructions_offset = Vec2::new(0.0, -logo_size.y - 44.0);
    let header_offset = Vec2::new(0.0, -logo_size.y);
    let paper_top_left = Vec2::new(
        paper_position.x - paper_size.x / 2.0,
        paper_position.y + paper_size.y / 2.0,
    );
    let paper_top_middle = Vec2::new(paper_position.x, paper_top_left.y);

    let logo = commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("cia.png"),
                sprite: Sprite {
                    custom_size: Some(logo_size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    // Horizontally centered on the page
                    paper_position.x,
                    paper_position.y + paper_size.y / 2.0 - logo_size.y / 2.0,
                    21.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let text_header = commands
        .spawn((
            Text2dBundle {
                text: Text::from_section(
                    "Analyst Handbook\nChapter 10: Intervention Policy".to_owned(),
                    TextStyle {
                        font: mono_font.clone(),
                        font_size: 18.0,
                        color: Color::BLACK,
                    },
                )
                .with_alignment(TextAlignment::TOP_CENTER),
                transform: Transform::from_translation(
                    (paper_top_middle + header_offset).extend(21.0),
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let text_instructions = commands
        .spawn((
            Text2dBundle {
                text: Text::from_section(
                    "  1. Assassinate if interfering with U.S. corporate involvement or oil import
  2. Assassinate if providing high quality of life to citizens
  3. If constituents demand replacement, select suitable replacement, then perform coup"
                        .to_owned(),
                    TextStyle {
                        font: mono_font,
                        font_size: 14.0,
                        color: Color::BLACK,
                    },
                ),
                text_2d_bounds: Text2dBounds {
                    size: Vec2::new(
                        paper_size.x - instructions_offset.x,
                        paper_size.y + instructions_offset.y,
                    ),
                },
                transform: Transform::from_translation(
                    (paper_top_left + instructions_offset).extend(21.0),
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(paper_size),
                ..default()
            },
            transform: Transform::from_translation(paper_position.extend(20.0)),
            ..default()
        },
        DragHitBox {
            size: paper_size,
            members: vec![text_header, text_instructions, logo],
        },
        OnDesk,
    ));
}

fn spawn_dossier(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paper_size = Vec2::new(350.0, 350.0 * 11.0 / 8.5);
    let paper_position = Vec2::new(-150.0, 50.0);

    // Dossier - the description of the person
    // - Name
    // - Photo
    // - Country
    // - List of notable policies
    // - A spot for your stamp for your decision

    let logo_size = Vec2::new(80.0, 80.0);

    let logo = commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("cia.png"),
                sprite: Sprite {
                    custom_size: Some(logo_size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    paper_position.x - paper_size.x / 2.0 + logo_size.x / 2.0,
                    paper_position.y + paper_size.y / 2.0 - logo_size.y / 2.0,
                    41.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let headshot_size = Vec2::new(90.0, 90.0 * 11.0 / 8.5);

    let margin = 4.0;

    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let headshot = commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("nasser.png"),
                sprite: Sprite {
                    custom_size: Some(headshot_size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    paper_position.x - paper_size.x / 2.0 + headshot_size.x / 2.0,
                    paper_position.y + paper_size.y / 2.0
                        - logo_size.y
                        - headshot_size.y / 2.0
                        - margin,
                    41.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let text_confidential = commands
        .spawn((
            Text2dBundle {
                text: Text::from_section(
                    "DO NOT COPY/CONFIDENTIAL\nForeign Leader Report".to_owned(),
                    TextStyle {
                        font: mono_font.clone(),
                        font_size: 18.0,
                        color: Color::BLACK,
                    },
                ),
                transform: Transform::from_xyz(
                    paper_position.x - paper_size.x / 2.0 + logo_size.x + margin,
                    paper_position.y + paper_size.y / 2.0 - margin,
                    41.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let text_facts = commands
        .spawn((
            Text2dBundle {
                text: Text::from_section(
                    "\
Name: Gamal Abdel Nasser
Title: President of Egypt
D.O.B.: 15 January 1918
Gender: Male
Nationality: Egypt
Constituency: Loyal
Eye Color: Brown
Hair Color: Black"
                        .to_owned(),
                    TextStyle {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        color: Color::BLACK,
                    },
                ),
                transform: Transform::from_xyz(
                    paper_position.x - paper_size.x / 2.0 + headshot_size.x + margin,
                    paper_position.y + paper_size.y / 2.0 - logo_size.y,
                    41.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let text_policies = commands
        .spawn((
            Text2dBundle {
                text: Text::from_section(
                    "\
Policies:
• Universal Health Care
• Free education
• Redistributes land to small farmers
• Nationalizes local industry
• No foreign corporations in Egypt
• Interefered with global oil import"
                        .to_owned(),
                    TextStyle {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        color: Color::BLACK,
                    },
                ),
                transform: Transform::from_xyz(
                    paper_position.x - paper_size.x / 2.0 + margin,
                    paper_position.y + paper_size.y / 2.0
                        - logo_size.y
                        - headshot_size.y
                        - margin * 2.0,
                    41.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    let stamped_size_y = 100.0;
    let text_stamp_label = commands
        .spawn((
            Text2dBundle {
                text: Text::from_section(
                    "Analyst Recommendation:\n(Place Stamp Below)".to_owned(),
                    TextStyle {
                        font: mono_font,
                        font_size: 15.0,
                        color: Color::BLACK,
                    },
                )
                .with_alignment(TextAlignment::BOTTOM_CENTER),
                transform: Transform::from_xyz(
                    paper_position.x,
                    paper_position.y - paper_size.y / 2.0 + stamped_size_y + margin,
                    41.0,
                ),
                ..default()
            },
            OnDesk,
        ))
        .id();

    // One option
    // Just have all the parts of the dossier as separate entities with a shared component and iterate over them when starting to drag
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(paper_size),
                ..default()
            },
            transform: Transform::from_translation(paper_position.extend(40.0)),
            ..default()
        },
        DragHitBox {
            size: paper_size,
            members: vec![
                text_facts,
                logo,
                headshot,
                text_confidential,
                text_stamp_label,
                text_policies,
            ],
        },
        Dossier,
        OnDesk,
    ));
}

fn setup(mut commands: Commands) {
    commands.init_resource::<Mouse>();
    commands.init_resource::<StampStatus>();
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_stamp(mut commands: Commands, asset_server: Res<AssetServer>) {
    let stamp_size = Vec2::new(150.0, 150.0);
    let stamped_size = Vec2::new(150.0, 100.0);
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("AssassinateStamp.png"),
            sprite: Sprite {
                custom_size: Some(stamp_size),
                ..default()
            },
            transform: Transform::from_xyz(400.0, -200.0, 61.0),
            ..default()
        },
        DragHitBox {
            size: stamp_size,
            members: vec![],
        },
        Stamp {
            stamped_sprite: SpriteBundle {
                texture: asset_server.load("AssassinateStamped.png"),
                sprite: Sprite {
                    custom_size: Some(stamped_size),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, -20.0, 60.0),
                ..default()
            },
        },
        OnDesk,
    ));
}

#[derive(Debug, Component)]
struct DragHitBox {
    size: Vec2,
    members: Vec<Entity>,
}

#[derive(Component)]
struct BeingDragged;

fn begin_being_dragged(
    input: Res<Input<MouseButton>>,
    mouse: Res<Mouse>,
    query: Query<(Entity, &DragHitBox, &GlobalTransform, Option<&Stamp>)>,
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut stamp_status: ResMut<StampStatus>,
) {
    if input.just_pressed(MouseButton::Left) {
        let front_clicked_entity = query
            .iter()
            .filter(|&(_, hitbox, transform, _)| hovers(hitbox, transform, &mouse))
            .max_by_key(|(_, _, transform, _)| NotNan::new(transform.translation().z).unwrap());

        if let Some((entity, hitbox, _, stamp)) = front_clicked_entity {
            windows
                .get_primary_mut()
                .unwrap()
                .set_cursor_icon(CursorIcon::Grabbing);
            commands.entity(entity).insert(BeingDragged);
            for &member in &hitbox.members {
                commands.entity(member).insert(BeingDragged);
            }

            if matches!(*stamp_status, StampStatus::Dropped) && stamp.is_some() {
                dbg!("picked_up");
                *stamp_status = StampStatus::PickedUp(Timer::from_seconds(5.0, TimerMode::Once))
            }
        }
    }
}

fn hovers(hitbox: &DragHitBox, transform: &GlobalTransform, mouse: &Mouse) -> bool {
    mouse.position.x > transform.translation().x - hitbox.size.x * 0.5
        && mouse.position.x < transform.translation().x + hitbox.size.x * 0.5
        && mouse.position.y > transform.translation().y - hitbox.size.y * 0.5
        && mouse.position.y < transform.translation().y + hitbox.size.y * 0.5
}

#[derive(Component)]
struct Stamp {
    stamped_sprite: SpriteBundle,
}

#[derive(Component)]
struct Dossier;

#[derive(Default, Resource)]
enum StampStatus {
    #[default]
    Initial,
    Dropped,
    PickedUp(Timer),
}

fn stop_being_dragged(
    input: Res<Input<MouseButton>>,
    being_dragged: Query<(Entity, Option<(&Stamp, &Transform)>), With<BeingDragged>>,
    mut dossier: Query<(&mut DragHitBox, &Transform), With<Dossier>>,
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut stamp_status: ResMut<StampStatus>,
) {
    if input.just_released(MouseButton::Left) {
        windows
            .get_primary_mut()
            .unwrap()
            .set_cursor_icon(CursorIcon::Default);
        for (entity, stamp) in &being_dragged {
            commands.entity(entity).remove::<BeingDragged>();

            // Dropping the stamp means stamping it
            if let Some((stamp, stamp_transform)) = stamp {
                let mut stamped_sprite = stamp.stamped_sprite.clone();
                stamped_sprite.transform.translation.x += stamp_transform.translation.x;
                stamped_sprite.transform.translation.y += stamp_transform.translation.y;

                let (mut hitbox, dossier_transform) = dossier.single_mut();
                if stamp_fits(
                    stamp,
                    &stamped_sprite.transform,
                    dossier_transform,
                    hitbox.size,
                ) {
                    let id = commands.spawn((stamped_sprite, OnDesk)).id();
                    hitbox.members.push(id);

                    if matches!(*stamp_status, StampStatus::Initial) {
                        *stamp_status = StampStatus::Dropped;
                    }
                }
            }
        }
    }
}

fn stamp_fits(
    stamp: &Stamp,
    stamped_transform: &Transform,
    dossier_transform: &Transform,
    dossier_size: Vec2,
) -> bool {
    let stamped = Rect::from_center_size(
        stamped_transform.translation.truncate(),
        stamp.stamped_sprite.sprite.custom_size.unwrap(),
    );
    let dossier = Rect::from_center_size(dossier_transform.translation.truncate(), dossier_size);
    fits(dossier, stamped)
}

fn fits(outer: Rect, inner: Rect) -> bool {
    outer.min.cmple(inner.min).all() && inner.max.cmple(outer.max).all()
}

fn drag(mouse: Res<Mouse>, mut query: Query<&mut Transform, With<BeingDragged>>) {
    if mouse.is_changed() {
        for mut transform in &mut query {
            transform.translation.x += mouse.position_delta.x;
            transform.translation.y += mouse.position_delta.y;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Desk,
    Newspaper,
}

fn check_timer(mut stamp_status: ResMut<StampStatus>, time: Res<Time>, mut commands: Commands) {
    if let StampStatus::PickedUp(timer) = &mut *stamp_status {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.insert_resource(NextState(GameState::Newspaper))
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct Mouse {
    /// Position in world coordinates.
    pub position: Vec2,
    /// Position in logical pixels in the window.
    pub screen_position: Vec2,
    /// Position in logical pixels in the window inverted (needed for UI)
    pub screen_pos_inverted: Vec2,
    pub out_of_bounds: bool,

    pub position_delta: Vec2,
}

/// References
/// 1. calc_mouse_pos
/// https://bevy-cheatbook.github.io/cookbook/cursor2world.html
///
/// Runs on a separate stage before everything else.
fn calc_mouse_pos(
    windows: Res<Windows>,
    mut mouse: ResMut<Mouse>,
    query_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let Ok((camera, camera_transform)) = query_cam.get_single() {
        // Bevy will not return anything here if the mouse is out of screen bounds...
        // ... unless a mouse button is pressed, for whatever reason.
        // That's why there's a double check for mouse being out of bounds.
        let window = windows.get_primary().unwrap();
        if let Some(screen_pos) = window.cursor_position() {
            let window_size = Vec2::new(window.width(), window.height());
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE; // What the heck does ndc stand for?
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            let world_position = ndc_to_world.project_point3(ndc.extend(-1.0));
            let world_position: Vec2 = world_position.truncate();

            let old_position = mem::replace(&mut mouse.position, world_position);
            mouse.screen_position = screen_pos;
            mouse.screen_pos_inverted = Vec2::new(screen_pos.x, window.height() - screen_pos.y);
            mouse.out_of_bounds = screen_pos.x < 0.
                || screen_pos.x > window.width()
                || screen_pos.y < 0.
                || screen_pos.y > window.height();
            mouse.position_delta = mouse.position - old_position;
        } else {
            mouse.out_of_bounds = true;
        }
    }
}
