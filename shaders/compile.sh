#!/usr/bin/env bash
set -euo pipefail

mkdir -p spirv
glslc triangle.vert -o spirv/triangle.vert.spv
glslc triangle.frag -o spirv/triangle.frag.spv
echo "OK: compiled shaders to shaders/spirv/"

