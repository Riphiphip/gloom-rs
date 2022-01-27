#version 430 core

out vec4 color;
in vec4 vertex_color;

uniform vec2 screenDims;
uniform float iTime;

void main()
{

    color = vertex_color;
}