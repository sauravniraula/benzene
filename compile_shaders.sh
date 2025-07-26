#!/bin/bash

SHADER_DIR="src/shaders"
OUTPUT_DIR="src/shaders"

if ! command -v glslc &>/dev/null; then
    echo "Error: glslc not found. Please install Vulkan SDK or shaderc-tools."
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

compile_shader() {
    local input_file="$1"
    local output_file="${input_file}.spv"
    echo "Compiling $input_file -> $output_file"
    if glslc "$input_file" -o "$output_file"; then
        echo "✓ Successfully compiled $input_file"
    else
        echo "✗ Failed to compile $input_file"
        return 1
    fi
}

shader_files=($(find "$SHADER_DIR" -name "*.vert" -o -name "*.frag" -o -name "*.comp" -o -name "*.geom" -o -name "*.tesc" -o -name "*.tese"))

if [ ${#shader_files[@]} -eq 0 ]; then
    echo "No shader files found in $SHADER_DIR"
    exit 0
fi

echo "Found ${#shader_files[@]} shader file(s):"
for file in "${shader_files[@]}"; do
    echo "  - $file"
done
echo

failed_compilations=0
for shader_file in "${shader_files[@]}"; do
    if ! compile_shader "$shader_file"; then
        ((failed_compilations++))
    fi
done

echo
if [ $failed_compilations -eq 0 ]; then
    echo "✓ All shaders compiled successfully!"
else
    echo "✗ $failed_compilations shader(s) failed to compile"
    exit 1
fi
