#version 430 core

out vec4 color;
layout(location=0) uniform uvec2 screenDims;
uniform float iTime;
void main()
{
    float normalize_factor = float(max(screenDims.x,screenDims.x));
    float tile_size = 0.01;
    vec2 uv = vec2(float(gl_FragCoord.x)/normalize_factor, float(gl_FragCoord.y)/normalize_factor);

    mat2 rot_mat = mat2(
        cos(radians(45)), -sin(radians(45)),
        sin(radians(45)), cos(radians(45))
    );

    uv = rot_mat * uv;

    int x_stripe_number = int(floor(uv.x/tile_size));
    int y_stripe_number = int(floor(uv.y/tile_size));

    bool is_white = bool(mod(x_stripe_number,2)) ^^ bool(mod(y_stripe_number,2));

    color = vec4(is_white,is_white,is_white, 1.0f);
}