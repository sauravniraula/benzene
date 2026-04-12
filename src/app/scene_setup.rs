use benzene::{
    Camera, CameraBundle, DirectionalLight, DirectionalLightBundle, Engine, MeshInstance,
    MeshInstanceBundle, PointLight, PointLightBundle, Result, Scene, Transform,
};
use nalgebra::{Vector3, Vector4};

pub(super) fn build_scene(engine: &mut Engine) -> Result<Scene> {
    let mut scene = engine.create_scene();

    let ground_texture = engine.load_texture("assets/textures/cracked-dirt512x512.jpg")?;
    let leaves_texture = engine.load_texture("assets/textures/grass/color.jpg")?;
    let light_texture = engine.load_texture("assets/textures/marble/color.jpg")?;

    let ground_material = engine.create_material(ground_texture)?;
    let trunk_material = engine.create_material(ground_texture)?;
    let leaves_material = engine.create_material(leaves_texture)?;
    let light_material = engine.create_material(light_texture)?;

    let ground_mesh = engine.load_mesh_obj("assets/models/plane.obj")?;
    let trunk_mesh = engine.load_mesh_obj("assets/models/vase-smooth.obj")?;
    let canopy_mesh = engine.load_mesh_obj("assets/models/torus-smooth.obj")?;

    let camera = scene.spawn_camera(
        CameraBundle::new(
            Transform::new(
                Vector3::new(0.0, 2.8, 6.5),
                Vector3::new((-12.0f32).to_radians(), 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
            Camera::default(),
        )
        .named("Camera"),
    );
    scene.set_active_camera(camera);

    scene.spawn_directional_light(
        DirectionalLightBundle::new(
            Transform::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new((-40.0f32).to_radians(), (-35.0f32).to_radians(), 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
            DirectionalLight::new(Vector4::new(0.95, 0.98, 1.0, 0.18)),
        )
        .named("Fill Light"),
    );

    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(0.0, 0.0, -5.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(4.0, 1.0, 4.0),
            ),
            MeshInstance::new(ground_mesh, Some(ground_material)),
        )
        .named("Ground"),
    );

    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(0.0, 0.0, -5.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.35, 1.25, 0.35),
            ),
            MeshInstance::new(trunk_mesh, Some(trunk_material)),
        )
        .named("Tree Trunk"),
    );

    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(0.0, 2.05, -5.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.95, 0.55, 0.95),
            ),
            MeshInstance::new(canopy_mesh, Some(leaves_material)),
        )
        .named("Canopy Base"),
    );

    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(0.0, 2.65, -5.0),
                Vector3::new(0.0, 0.7, 0.0),
                Vector3::new(0.72, 0.42, 0.72),
            ),
            MeshInstance::new(canopy_mesh, Some(leaves_material)),
        )
        .named("Canopy Mid"),
    );

    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(0.0, 3.15, -5.0),
                Vector3::new(0.0, 1.2, 0.0),
                Vector3::new(0.48, 0.30, 0.48),
            ),
            MeshInstance::new(canopy_mesh, Some(leaves_material)),
        )
        .named("Canopy Top"),
    );

    scene.spawn_point_light(
        PointLightBundle::new(
            Transform::new(
                Vector3::new(2.2, 3.6, -3.2),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.16, 0.08, 0.16),
            ),
            PointLight::new(Vector4::new(1.0, 0.92, 0.75, 20.0)),
        )
        .named("Lamp"),
    );
    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(2.2, 3.6, -3.2),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.16, 0.08, 0.16),
            ),
            MeshInstance::new(canopy_mesh, Some(light_material)),
        )
        .named("Lamp Mesh"),
    );

    Ok(scene)
}
