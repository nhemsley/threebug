use std::net::SocketAddr;

use bevy::{
    // diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseWheel,
    pbr::wireframe::WireframePlugin,
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    window::CursorGrabMode,
};

use bevy_egui::{EguiContext, EguiPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_spicy_networking::*;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    // controllers::fps_3d::{Fps3dCameraBundle, Fps3dCameraController, Fps3dCameraPlugin},
    LookTransformPlugin,
};

use threebug_core::{ipc::DebugEntity, EntityRegistry};
use threebug_server::{
    render::sessions::SessionsState,
    resource::session::{Session, Sessions},
};

use threebug_server::ui;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugin(WireframePlugin)
        // bevy spicy networking
        .add_plugin(bevy_spicy_networking::ServerPlugin)
        // smooth bevy cameras
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        // bevy egui
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(ui::ui)
        .add_startup_system(setup)
        .add_system(fps)
        .add_system(cursor_grab_system)
        .add_system(render);

    // Register parry server messages
    register_server_network_messages(&mut app);
    app.add_startup_system(setup_networking)
        .add_system(handle_connection_events)
        .add_system(handle_messages);

    app.insert_resource(Sessions::default());
    app.insert_resource(SessionsState::default());
    app.insert_resource(EntityRegistry::default());

    app.run();
}

fn setup_networking(mut net: ResMut<NetworkServer>) {
    let ip_address = "127.0.0.1".parse().expect("Could not parse ip address");

    let socket_address = SocketAddr::new(ip_address, 9876);

    info!("Address of the server: {}", socket_address);

    match net.listen(socket_address) {
        Ok(_) => (),
        Err(err) => {
            error!("Could not start listening: {}", err);
            panic!();
        }
    }

    info!("Started listening for new connections!");
}

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
        ));
}
fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sessions: ResMut<Sessions>,
    mut session_render_state: ResMut<SessionsState>,
) {
    if session_render_state.is_current(&*sessions) {
        let mut clean_session = false;
        if let Some(session) = sessions.current_session() {
            if session.entities.is_dirty() {
                clean_session = true;
                info!("session is dirty");
                session_render_state.spawn_current_session(
                    &mut *sessions,
                    &mut commands,
                    &mut *meshes,
                    &mut *materials,
                    false,
                );
            }
        }
        if clean_session {
            if let Some(session) = sessions.current_session_mut() {
                session.entities.clean();
            }
        }
    } else {
        info!("session is not current, despawning");
        session_render_state.despawn_current_session(
            &mut *sessions,
            &mut commands,
            &mut *meshes,
            &mut *materials,
        );
        info!("updating current session");

        session_render_state.update_current_session(&*sessions);
        info!("spawning new session");

        session_render_state.spawn_current_session(
            &mut *sessions,
            &mut commands,
            &mut *meshes,
            &mut *materials,
            true,
        );

        if let Some(session) = sessions.current_session_mut() {
            session.entities.clean();
        }
    }
}

fn handle_connection_events(
    mut _commands: Commands,
    _net: Res<NetworkServer>,
    mut network_events: EventReader<ServerNetworkEvent>,
    mut sessions: ResMut<Sessions>,
) {
    // info!("handle_connection_events");
    for event in network_events.iter() {
        if let ServerNetworkEvent::Connected(conn_id) = event {
            let session = Session::new(*conn_id);
            sessions.insert(session);

            //TODO: send accept accepted to client
            info!("New client connected: {}", conn_id);
        }
    }
}

fn handle_messages(
    mut new_messages: EventReader<NetworkData<threebug_core::ipc::DebugEntity>>,
    // net: Res<NetworkServer>,
    mut sessions: ResMut<Sessions>,
    mut entity_registry: ResMut<EntityRegistry>,
) {
    let mut session_len = 0;
    for message in new_messages.iter() {
        let session_id = &message.source().uuid().to_string();
        if let Some(session) = sessions.get_mut(session_id) {
            info!("Got session for: {}", session.id());

            let mut entity = (*message).clone();
            entity_registry.assign_id(&mut entity.id);

            info!("New Entity: {:?}", entity);

            session.entities.push(entity.clone());
            session_len = session.entities.len();
        }
        info!("{} entities", session_len);
    }
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut controllers: Query<&mut FpsCameraController>,
    mut ui_context: ResMut<EguiContext>,
) {
    let window = windows.get_primary_mut().unwrap();

    let mut controller = controllers.get_single_mut().unwrap();

    // we want to be able to catch Esc keys, even if ctx().is_pointer_over_area()
    if key.just_pressed(KeyCode::Escape) {
        info!("disabling fps 3d controller");
        controller.enabled = false;
        window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Locked);
        window.set_cursor_visibility(true);
    }

    if ui_context.ctx_mut().is_pointer_over_area() {
        return;
    }

    // but we dont want to respond to left mouse clicks
    if btn.just_pressed(MouseButton::Left) {
        info!("enabling fps 3d controller");

        controller.enabled = true;
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        window.set_cursor_visibility(false);
    }
}

fn fps(
    _keyboard: Res<Input<KeyCode>>,
    _mouse: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    _time: Res<Time>,
    mut fps: Query<&mut FpsCameraController>,
) {
    if let Ok(mut fps) = fps.get_single_mut() {
        for event in mouse_wheel_events.iter() {
            //FIXME: move this into some kind of easing function thingy
            let delta = if fps.translate_sensitivity <= 0.2 {
                0.01
            } else if fps.translate_sensitivity <= 1.0 {
                0.1
            } else {
                0.3
            };
            let delta = event.y * delta;
            fps.translate_sensitivity += delta;
            fps.translate_sensitivity = fps.translate_sensitivity.clamp(0.01, 10.0);
            info!(
                "Changing translate sensitivity by {} to {}",
                delta, fps.translate_sensitivity
            );
        }
    }
}

pub fn register_server_network_messages(app: &mut App) {
    app.listen_for_server_message::<DebugEntity>();
}
