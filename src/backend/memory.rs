pub fn find_memory_type_index(
    memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
    type_bits: u32,
    flags: ash::vk::MemoryPropertyFlags,
) -> u32 {
    let mut memory_index: Option<u32> = None;
    let memory_types = memory_properties.memory_types;
    for (idx, each) in memory_types.iter().enumerate() {
        let type_found = type_bits & (1 << idx) > 0;
        let properties_found = each.property_flags.contains(flags);
        if type_found && properties_found {
            memory_index = Some(idx as u32);
        }
    }
    memory_index.expect("unable to find suitable memory type")
}
