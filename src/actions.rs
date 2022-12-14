use crate::GameState;
use bevy::{prelude::*, render::camera::RenderTarget};

/// This plugin listens for keyboard input and converts the input into Actions
/// Actions can then be used as a resource in other systems to act on the player input.
pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>().add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(set_movement_actions)
                .with_system(cursor_system)
                .with_system(set_item_pick_up)
                .with_system(set_interact)
                .with_system(use_item),
        );
    }
}

/// Possible input actions
#[derive(Default, Resource)]
pub struct Actions {
    /// moving the player
    pub player_movement: Option<Vec2>,
    /// Mouse rollover
    pub mouse_rollover: Option<MousePosition>,
    /// Pick up item
    pub pick_up_item: Option<bool>,
    /// Interact
    pub interact: Option<bool>,
    /// Use an item in inventory
    pub use_item: Option<usize>,
}

/// Position of the mouse cursor
#[derive(Default, Debug)]
pub struct MousePosition {
    /// In the game
    pub game_position: Vec2,
    /// on the screen
    pub screen_position: Vec2,
}

/// Code the convert cursor location to game location
fn cursor_system(
    mut actions: ResMut<Actions>,
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width(), wnd.height());

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        actions.mouse_rollover = Some(MousePosition {
            game_position: world_pos,
            screen_position: screen_pos,
        });
    } else {
        actions.mouse_rollover = None;
    }
}

/// From keyboard input turn into player movement
fn set_movement_actions(
    mut actions: ResMut<Actions>,
    mut mut_keyboard_input: ResMut<Input<KeyCode>>,
) {
    let keyboard_input = mut_keyboard_input.as_ref();
    if GameControl::Up.just_released(keyboard_input)
        || GameControl::Up.just_pressed(keyboard_input)
        || GameControl::Left.just_released(keyboard_input)
        || GameControl::Left.just_pressed(keyboard_input)
        || GameControl::Down.just_released(keyboard_input)
        || GameControl::Down.just_pressed(keyboard_input)
        || GameControl::Right.just_released(keyboard_input)
        || GameControl::Right.just_pressed(keyboard_input)
    {
        let mut player_movement = Vec2::ZERO;

        if GameControl::Up.just_released(keyboard_input)
            || GameControl::Down.just_released(keyboard_input)
        {
            if GameControl::Up.pressed(keyboard_input) {
                player_movement.y = 1.;
            } else if GameControl::Down.pressed(keyboard_input) {
                player_movement.y = -1.;
            } else {
                player_movement.y = 0.;
            }
        } else if GameControl::Up.just_pressed(keyboard_input) {
            player_movement.y = 1.;
        } else if GameControl::Down.just_pressed(keyboard_input) {
            player_movement.y = -1.;
        } else {
            player_movement.y = actions.player_movement.unwrap_or(Vec2::ZERO).y;
        }

        if GameControl::Right.just_released(keyboard_input)
            || GameControl::Left.just_released(keyboard_input)
        {
            if GameControl::Right.pressed(keyboard_input) {
                player_movement.x = 1.;
            } else if GameControl::Left.pressed(keyboard_input) {
                player_movement.x = -1.;
            } else {
                player_movement.x = 0.;
            }
        } else if GameControl::Right.just_pressed(keyboard_input) {
            player_movement.x = 1.;
        } else if GameControl::Left.just_pressed(keyboard_input) {
            player_movement.x = -1.;
        } else {
            player_movement.x = actions.player_movement.unwrap_or(Vec2::ZERO).x;
        }

        if player_movement != Vec2::ZERO {
            player_movement = player_movement.normalize();
            info!("Keyboard input made player movement: {}", player_movement);
            actions.player_movement = Some(player_movement);
            mut_keyboard_input.clear();
        }
    } else {
        actions.player_movement = None;
    }
}

/// From keyboard input turn into item pick up
fn set_item_pick_up(mut actions: ResMut<Actions>, mut mut_keyboard_input: ResMut<Input<KeyCode>>) {
    let keyboard_input = mut_keyboard_input.as_ref();
    if GameControl::PickUp.just_released(keyboard_input)
        || GameControl::PickUp.just_pressed(keyboard_input)
    {
        actions.pick_up_item = Some(true);
        info!("Keyboard input made player pick up");
        mut_keyboard_input.clear();
    } else {
        actions.pick_up_item = None;
    }
}

/// From keyboard input turn into item pick up
fn set_interact(mut actions: ResMut<Actions>, mut mut_keyboard_input: ResMut<Input<KeyCode>>) {
    let keyboard_input = mut_keyboard_input.as_ref();
    if GameControl::Interact.just_released(keyboard_input)
        || GameControl::Interact.just_pressed(keyboard_input)
    {
        actions.interact = Some(true);
        info!("Keyboard input made player interact");
        mut_keyboard_input.clear();
    } else {
        actions.interact = None;
    }
}

/// From keyboard input turn into player inventory choice
fn use_item(mut actions: ResMut<Actions>, mut mut_keyboard_input: ResMut<Input<KeyCode>>) {
    let keyboard_input = mut_keyboard_input.as_ref();
    let mut used = false;
    (0..9).for_each(|n| {
        if GameControl::UseItem(n).just_released(keyboard_input)
            || GameControl::UseItem(n).just_pressed(keyboard_input)
        {
            actions.use_item = Some(n);
            info!("Keyboard input made player use item {}", n);
            used = true;
        }
    });
    if used {
        mut_keyboard_input.clear();
    } else {
        actions.use_item = None;
    }
}

/// Possible Player actions
enum GameControl {
    /// Move up
    Up,
    /// Move down
    Down,
    /// Move left
    Left,
    /// Move right
    Right,
    /// Pick up Item
    PickUp,
    /// Interact button to cover multiple options
    Interact,
    /// Use Item
    UseItem(usize),
}

impl GameControl {
    /// Convert keyboard input to game control on released
    fn just_released(&self, keyboard_input: &Input<KeyCode>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_released(KeyCode::W)
                    || keyboard_input.just_released(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.just_released(KeyCode::S)
                    || keyboard_input.just_released(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.just_released(KeyCode::A)
                    || keyboard_input.just_released(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.just_released(KeyCode::D)
                    || keyboard_input.just_released(KeyCode::Right)
            }
            GameControl::PickUp => keyboard_input.just_released(KeyCode::G),
            GameControl::Interact => keyboard_input.just_released(KeyCode::E),
            GameControl::UseItem(0) => keyboard_input.just_released(KeyCode::Key0),
            GameControl::UseItem(1) => keyboard_input.just_released(KeyCode::Key1),
            GameControl::UseItem(2) => keyboard_input.just_released(KeyCode::Key2),
            GameControl::UseItem(3) => keyboard_input.just_released(KeyCode::Key3),
            GameControl::UseItem(4) => keyboard_input.just_released(KeyCode::Key4),
            GameControl::UseItem(5) => keyboard_input.just_released(KeyCode::Key5),
            GameControl::UseItem(6) => keyboard_input.just_released(KeyCode::Key6),
            GameControl::UseItem(7) => keyboard_input.just_released(KeyCode::Key7),
            GameControl::UseItem(8) => keyboard_input.just_released(KeyCode::Key8),
            GameControl::UseItem(9) => keyboard_input.just_released(KeyCode::Key9),
            GameControl::UseItem(_) => false,
        }
    }

    /// Convert keyboard input to game control on pressed
    fn pressed(&self, keyboard_input: &Input<KeyCode>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right)
            }
            GameControl::PickUp => keyboard_input.pressed(KeyCode::G),
            GameControl::Interact => keyboard_input.pressed(KeyCode::E),
            GameControl::UseItem(0) => keyboard_input.pressed(KeyCode::Key0),
            GameControl::UseItem(1) => keyboard_input.pressed(KeyCode::Key1),
            GameControl::UseItem(2) => keyboard_input.pressed(KeyCode::Key2),
            GameControl::UseItem(3) => keyboard_input.pressed(KeyCode::Key3),
            GameControl::UseItem(4) => keyboard_input.pressed(KeyCode::Key4),
            GameControl::UseItem(5) => keyboard_input.pressed(KeyCode::Key5),
            GameControl::UseItem(6) => keyboard_input.pressed(KeyCode::Key6),
            GameControl::UseItem(7) => keyboard_input.pressed(KeyCode::Key7),
            GameControl::UseItem(8) => keyboard_input.pressed(KeyCode::Key8),
            GameControl::UseItem(9) => keyboard_input.pressed(KeyCode::Key9),
            GameControl::UseItem(_) => false,
        }
    }

    /// Convert keyboard input to game control on just pressed
    fn just_pressed(&self, keyboard_input: &Input<KeyCode>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.just_pressed(KeyCode::S)
                    || keyboard_input.just_pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.just_pressed(KeyCode::A)
                    || keyboard_input.just_pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.just_pressed(KeyCode::D)
                    || keyboard_input.just_pressed(KeyCode::Right)
            }
            GameControl::PickUp => keyboard_input.just_pressed(KeyCode::G),
            GameControl::Interact => keyboard_input.just_pressed(KeyCode::E),
            GameControl::UseItem(0) => keyboard_input.just_pressed(KeyCode::Key0),
            GameControl::UseItem(1) => keyboard_input.just_pressed(KeyCode::Key1),
            GameControl::UseItem(2) => keyboard_input.just_pressed(KeyCode::Key2),
            GameControl::UseItem(3) => keyboard_input.just_pressed(KeyCode::Key3),
            GameControl::UseItem(4) => keyboard_input.just_pressed(KeyCode::Key4),
            GameControl::UseItem(5) => keyboard_input.just_pressed(KeyCode::Key5),
            GameControl::UseItem(6) => keyboard_input.just_pressed(KeyCode::Key6),
            GameControl::UseItem(7) => keyboard_input.just_pressed(KeyCode::Key7),
            GameControl::UseItem(8) => keyboard_input.just_pressed(KeyCode::Key8),
            GameControl::UseItem(9) => keyboard_input.just_pressed(KeyCode::Key9),
            GameControl::UseItem(_) => false,
        }
    }
}
