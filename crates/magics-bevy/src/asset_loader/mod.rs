//! Asset loading functionality for the Magics planner UI
//!
//! This module provides asset loading and caching mechanisms for
//! textures, models, and other resources used by the UI.

use std::collections::HashMap;
use bevy::prelude::*;

/// Resource for managing loaded assets
#[derive(Resource, Default)]
pub struct AssetLibrary {
    /// Loaded texture assets
    textures: HashMap<String, Handle<Image>>,
    /// Loaded mesh assets
    meshes: HashMap<String, Handle<Mesh>>,
    /// Loaded material assets
    materials: HashMap<String, Handle<StandardMaterial>>,
    /// Loaded scene assets
    scenes: HashMap<String, Handle<Scene>>,
}

impl AssetLibrary {
    /// Get a texture by key
    pub fn get_texture(&self, key: &str) -> Option<&Handle<Image>> {
        self.textures.get(key)
    }
    
    /// Get a mesh by key
    pub fn get_mesh(&self, key: &str) -> Option<&Handle<Mesh>> {
        self.meshes.get(key)
    }
    
    /// Get a material by key
    pub fn get_material(&self, key: &str) -> Option<&Handle<StandardMaterial>> {
        self.materials.get(key)
    }
    
    /// Get a scene by key
    pub fn get_scene(&self, key: &str) -> Option<&Handle<Scene>> {
        self.scenes.get(key)
    }
    
    /// Add a texture
    pub fn add_texture(&mut self, key: String, handle: Handle<Image>) {
        self.textures.insert(key, handle);
    }
    
    /// Add a mesh
    pub fn add_mesh(&mut self, key: String, handle: Handle<Mesh>) {
        self.meshes.insert(key, handle);
    }
    
    /// Add a material
    pub fn add_material(&mut self, key: String, handle: Handle<StandardMaterial>) {
        self.materials.insert(key, handle);
    }
    
    /// Add a scene
    pub fn add_scene(&mut self, key: String, handle: Handle<Scene>) {
        self.scenes.insert(key, handle);
    }
}

/// Event to request loading a texture
#[derive(Event)]
pub struct LoadTextureEvent {
    /// Path to the texture file
    pub path: String,
    /// Key to store the texture under
    pub key: String,
}

/// Event to request loading a mesh
#[derive(Event)]
pub struct LoadMeshEvent {
    /// Path to the mesh file
    pub path: String,
    /// Key to store the mesh under
    pub key: String,
}

/// Event to request loading a scene
#[derive(Event)]
pub struct LoadSceneEvent {
    /// Path to the scene file
    pub path: String,
    /// Key to store the scene under
    pub key: String,
}

/// Plugin for asset loading functionality
pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AssetLibrary>()
            .add_event::<LoadTextureEvent>()
            .add_event::<LoadMeshEvent>()
            .add_event::<LoadSceneEvent>()
            .add_systems(Update, (
                handle_texture_loading,
                handle_mesh_loading,
                handle_scene_loading,
            ));
    }
}

/// System to handle texture loading requests
fn handle_texture_loading(
    mut events: EventReader<LoadTextureEvent>,
    mut asset_library: ResMut<AssetLibrary>,
    asset_server: Res<AssetServer>,
) {
    for event in events.iter() {
        let handle: Handle<Image> = asset_server.load(&event.path);
        asset_library.add_texture(event.key.clone(), handle);
        info!("Loaded texture: {}", event.path);
    }
}

/// System to handle mesh loading requests
fn handle_mesh_loading(
    mut events: EventReader<LoadMeshEvent>,
    mut asset_library: ResMut<AssetLibrary>,
    asset_server: Res<AssetServer>,
) {
    for event in events.iter() {
        let handle: Handle<Mesh> = asset_server.load(&event.path);
        asset_library.add_mesh(event.key.clone(), handle);
        info!("Loaded mesh: {}", event.path);
    }
}

/// System to handle scene loading requests
fn handle_scene_loading(
    mut events: EventReader<LoadSceneEvent>,
    mut asset_library: ResMut<AssetLibrary>,
    asset_server: Res<AssetServer>,
) {
    for event in events.iter() {
        let handle: Handle<Scene> = asset_server.load(&event.path);
        asset_library.add_scene(event.key.clone(), handle);
        info!("Loaded scene: {}", event.path);
    }
}

/// Load standard simulation assets
pub fn load_standard_assets(
    mut texture_events: EventWriter<LoadTextureEvent>,
    mut mesh_events: EventWriter<LoadMeshEvent>,
) {
    // Load standard textures
    texture_events.send(LoadTextureEvent {
        path: "textures/grid.png".to_string(),
        key: "grid".to_string(),
    });
    
    // Load standard meshes
    mesh_events.send(LoadMeshEvent {
        path: "meshes/agent.glb#Mesh0/Primitive0".to_string(),
        key: "agent".to_string(),
    });
}
