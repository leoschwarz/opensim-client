#version 140

uniform mat4 persp_matrix;
uniform mat4 view_matrix;

in vec3 position;
//in vec3 normal;
out vec3 v_position;
out vec3 v_normal;
out float v_color;

void main() {
    //v_position = position;
    //v_normal = normal;
    v_normal = vec3(1.0, 0.0, 0.0);
    gl_Position = persp_matrix * view_matrix * vec4(position, 1.0);
    v_color = position.z / 24.8;
}
