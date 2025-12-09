use crate::core::ecs::components::Structure3D;

// Placeholder: system functions for structures/assets
pub fn destroy_structure_3d(
    structure: &Structure3D,
    v_backend: &crate::vulkan_backend::backend::VBackend,
) {
    structure.destroy(v_backend);
}
