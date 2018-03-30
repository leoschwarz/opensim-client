#version 140

in vec3 v_normal;
in float v_color;
out vec4 f_color;

void main() {
    f_color = vec4(v_color, 0.5, v_color, 1.0);
}
