#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_coords;
layout(location=2) in vec4 a_color;
layout(location=3) in uint a_mode;

layout(location=0) out vec4 v_color;
layout(location=1) out vec2 v_coords;
layout(location=2) flat out uint v_mode;

void main() {
    v_color = a_color;
    v_coords = a_coords;
    v_mode = a_mode;
    gl_Position = vec4(a_position, 1.0);
}