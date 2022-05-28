use bevy::ecs::system::QuerySingleError;
use bevy::prelude::*;
use bevy::render::mesh::{PrimitiveTopology, Indices};

use crate::camera::MouseWorldPos;
use crate::game::components::selection::*;
use crate::game::layers_util;

pub fn click_select(
    ms_input: Res<Input<MouseButton>>,
    ms_pos: Res<MouseWorldPos>,
    query: Query<(Entity, &Transform), With<Selectable>>,
    mut ev_select_nearest: EventWriter<SelectNearest>,
) {
    if ms_input.just_pressed(MouseButton::Left) {
        let mut min_dist: f32 = 10.0;
        let mut min_entity: Option<Entity> = None;
        for (e, transform) in query.iter() {
            let mouse_pos_3d = layers_util::vec2_to_vec3(ms_pos.0, layers_util::Layers::Planets);
            let obj_dist = mouse_pos_3d.distance_squared(transform.translation);
            if obj_dist <= min_dist {
                min_dist = obj_dist;
                min_entity = Some(e);
            }
        }
        if let Some(entity) = min_entity {
            ev_select_nearest.send(SelectNearest(entity));
        }
    }
}

pub fn update_selected(
    mut commands: Commands,
    mut ev_select_nearest: EventReader<SelectNearest>,
    query: Query<Entity, With<Selected>>,
) {
    for ev in ev_select_nearest.iter() {
        for s_entity in query.iter() {
            commands.entity(s_entity).remove::<Selected>();
        }
        commands.entity(ev.0).insert(Selected);
        println!("Entity {:?} is now selected", ev.0);
    }
}

pub fn box_select(
    mut commands: Commands,
    ms_input: Res<Input<MouseButton>>,
    ms_pos: Res<MouseWorldPos>,
    selection_box: Query<Entity, With<SelectionBox>>,
    mut is_selecting_res: ResMut<IsSelecting>
) {
    if ms_input.just_pressed(MouseButton::Left){
        is_selecting_res.is_selecting = true;
        is_selecting_res.mouse_enter = Some(ms_pos.0);
    }
    if ms_input.just_released(MouseButton::Left) {
        is_selecting_res.is_selecting = false;
        is_selecting_res.mouse_enter = None;
        match selection_box.get_single() {
            Ok(e) =>commands.entity(e).despawn(),
            Err(_) => {}
        }
    }
}

pub fn draw_box_select(
    mut commands: Commands,
    is_selecting_res: Res<IsSelecting>,
    ms_pos: Res<MouseWorldPos>,
    mut query: Query<(&mut Handle<Mesh>, &mut Transform), With<SelectionBox>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>, 
) {
    if is_selecting_res.is_selecting {
        if let Some(ms_enter) = is_selecting_res.mouse_enter {
            match query.get_single_mut() {
                Ok((mut box_mesh, mut transform)) => {
                    *box_mesh = meshes.add(box_select_mesh(ms_enter, ms_pos.0));
                    transform.translation = layers_util::vec2_to_vec3(ms_enter, layers_util::Layers::BoxSelect);
                },
                Err(error) => {
                    match error {
                        QuerySingleError::NoEntities(_) => {
                            // let quad_handle = meshes.add(box_select_mesh(ms_enter, ms_pos.0));
                            let quad_handle = meshes.add(box_select_mesh(ms_enter, ms_pos.0));
                            let material_handle = materials.add(StandardMaterial {
                                base_color: Color::LIME_GREEN,
                                unlit: true,
                                ..default()
                            });
                            commands.spawn_bundle(PbrBundle {
                                mesh: quad_handle.clone(),
                                material: material_handle,
                                transform: Transform {
                                    translation: layers_util::vec2_to_vec3(ms_pos.0, layers_util::Layers::BoxSelect),
                                    ..default()
                                },
                                ..default()
                            }).insert(SelectionBox);
                        },
                        QuerySingleError::MultipleEntities(_) => panic!("Expected one entity at most")
                    };
                },
            }
        }
    }
}

pub fn box_select_mesh(screen_pos_origin: Vec2, screen_pos_target: Vec2) -> Mesh {
    let start = Vec2::ZERO;
    let end = screen_pos_target - screen_pos_origin;
    
    let top_left = Vec2::new(start.x.min(end.x), start.y.max(end.y));
    let bottom_right = Vec2::new(start.x.max(end.x), start.y.min(end.y));
    let bottom_left = Vec2::new(start.x.min(end.x), start.y.min(end.y));
    let top_right = Vec2::new(start.x.max(end.x), start.y.max(end.y));
    let z_value = layers_util::get_z(layers_util::Layers::BoxSelect);    

    // points are (vec3[position], vec2[uvs])
    // CHECK: UVs are random values
    let points = vec![
        ([bottom_left.x, bottom_left.y, z_value], [0.0, 1.0]),
        ([bottom_right.x, bottom_right.y, z_value], [1.0, 0.0]),
        ([top_left.x, top_left.y, z_value], [0.0, 1.0]),
        ([top_right.x, top_right.y, z_value], [1.0, 1.0]),
        ];
        let mut vertices = Vec::with_capacity(points.len());
    let mut uvs = Vec::with_capacity(points.len());
    let normals = vec![[0.0, 0.0, 1.0]; points.len()];

    for (position, uv) in points.iter() {
        vertices.push(*position);
        uvs.push(*uv);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2, 1, 3, 2])));
    mesh
}

