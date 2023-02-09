uniform float time;
uniform vec2 resolution;
uniform vec2 mouse;
uniform vec3 spectrum;
uniform mat4 mvp;

attribute vec4 a_position;
attribute vec3 a_normal;
attribute vec2 a_texcoord;

varying vec3 v_normal;
varying vec2 v_texcoord;

void main(void)
{
    gl_Position = a_position; 
    v_normal    = a_normal;
    v_texcoord  = vec2(a_texcoord.x,a_texcoord.y);
}

