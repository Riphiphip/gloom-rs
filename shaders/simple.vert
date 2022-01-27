#version 430 core

uniform float iTime;
uniform mat4 camera;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
out vec4 vertex_color;
void main()
{
    mat4 trans_mat = mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    vertex_color = color;
    gl_Position = camera * trans_mat * vec4(position, 1.0f);
}