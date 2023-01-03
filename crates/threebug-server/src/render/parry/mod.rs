use bevy::prelude::*;

use super::{MeshProvider, Spawnable};
use threebug_core::ipc::parry::*;

impl Spawnable for ParryDebugEntityType {
    fn spawn(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        match self {
            ParryDebugEntityType::AABB { aabb } => aabb.spawn(commands, meshes, materials),
        }
    }

    fn despawn(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        match self {
            ParryDebugEntityType::AABB { aabb } => aabb.despawn(commands, meshes, materials),
        }
    }
}

impl Spawnable for AABB {
    fn spawn(
        &mut self,
        commands: &mut bevy::prelude::Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        info!("spawning aabb");
        let mesh = meshes.add(self.mesh());
        let entity = commands
            .spawn(PbrBundle {
                mesh,
                material: materials.add(Color::rgba(0.3, 0.5, 0.3, 0.3).into()),
                ..Default::default()
            })
            // .insert(Wireframe)
            .id();
        self.entity = Some(entity.to_bits().into());
    }

    fn despawn(
        &mut self,
        commands: &mut bevy::prelude::Commands,
        _meshes: &mut Assets<Mesh>,
        _materials: &mut Assets<StandardMaterial>,
    ) {
        if let Some(entity) = self.entity {
            commands
                .entity(Entity::from_bits(entity.id))
                .despawn_recursive();
        }
    }
}

impl MeshProvider for AABB {
    fn mesh(&self) -> Mesh {
        let mins = self.aabb.mins;
        let maxs = self.aabb.maxs;
        let tbox = bevy::prelude::shape::Box {
            min_x: mins.x,
            min_y: mins.y,
            min_z: mins.z,
            max_x: maxs.x,
            max_y: maxs.y,
            max_z: maxs.z,
        };

        tbox.into()
    }
}
