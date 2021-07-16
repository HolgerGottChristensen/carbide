#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec2 v_tex_coords;
layout(location=2) flat in uint v_mode;

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;
layout(set = 0, binding = 2) uniform texture2D t_text_texture;
layout(set = 0, binding = 3) uniform texture2D atlas_texture;

void main() {
    if (v_mode == uint(0)) {
        float a = texture(sampler2D(atlas_texture, s_diffuse), v_tex_coords).a;
        f_color = vec4(v_color.r, v_color.g, v_color.b, a);
    } else if (v_mode == uint(1)) {
        f_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    } else if (v_mode == uint(2)) {
        f_color = v_color;
    } else if (v_mode == uint(3)) {
        f_color = texture(sampler2D(atlas_texture, s_diffuse), v_tex_coords);
    }
}