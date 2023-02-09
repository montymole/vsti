
#version 450
#extension GL_ARB_separate_shader_objects : enable

#ifdef GL_ES
precision highp float;
#endif

#define NEAR_CLIPPING_PLANE 1.0
#define FAR_CLIPPING_PLANE 40.0
#define NUMBER_OF_MARCH_STEPS 40
#define EPSILON 0.0001
#define DISTANCE_BIAS 1
#define PI 3.14159
#define PI2 2*PI

#define NUM_WAVES 8

//uniform float time;
//uniform vec2 resolution;
//uniform vec2 mouse;
//uniform vec3 spectrum;

//uniform sampler2D texture0;

//varying vec3 v_normal;
//varying vec2 textureCoord;


// Fragment shader that uses a texture coordinate to sample from a texture
// uniform.
layout(location = 0) in vec2 textureCoord;
layout(location = 1) out vec4 outColor;
//layout(location = 2) in float time;


layout(set = 0, binding = 1) uniform texture2D backgroundTexture;
layout(set = 0, binding = 2) uniform sampler textureSampler;

vec3 rotateX( in vec3 p, float t )
{
    float co = cos(t);
    float si = sin(t);
    p.yz = mat2(co,-si,si,co)*p.yz;
    return p;
}
vec3 rotateY( in vec3 p, float t )
{
    float co = cos(t);
    float si = sin(t);
    p.xz = mat2(co,-si,si,co)*p.xz;
    return p;
}
vec3 rotateZ( in vec3 p, float t )
{
    float co = cos(t);
    float si = sin(t);
    p.xy = mat2(co,-si,si,co)*p.xy;
    return p;
}

//https://github.com/marklundin/glsl-sdf-primitives

float sdBox( vec3 p, vec3 b )
{
  vec3 d = abs(p) - b;
  return min(max(d.x,max(d.y,d.z)),0.0) +
         length(max(d,0.0));
}

float sdTorus( vec3 p, vec2 t )
{
  vec2 q = vec2(length(p.xz)-t.x,p.y);
  return length(q)-t.y;
}

float sdHexPrism( vec3 p, vec2 h )
{
    vec3 q = abs(p);
    return max(q.z-h.y,max((q.x*0.866025+q.y*0.5),q.y)-h.x);
}

//https://www.alanzucconi.com/2016/07/01/signed-distance-functions/

float sdBlend(float d1, float d2, float a)
{
    return a * d1 + (1 - a) * d2;
}

float sdSmin(float a, float b, float k)
{
    float res = exp(-k*a) + exp(-k*b);
    return -log(max(0.0001,res)) / k;
}

// this generates animated shape
vec2 gui(in vec3 pos, vec3 mouse3d) {
    float time = 0.0;
    vec3 p = rotateX(pos+vec3(-.1,-1,0.5),time*.04);
    vec3 p2 = rotateZ(pos+vec3(.1,-1,0.5),time*.01);
    p = rotateY(p, time*0.1);
    float d1 = sdBox(p,vec3(.5));
    //float d2 = sdTorus(p2, vec2(1.,.1));
    float d2 = sdHexPrism(p2, vec2(.5,.3));
    //float d = sdBlend(d1,d2, 1.0+sin(time));
    float d = sdSmin(d1,d2,8);
    float m = 1.;
    return vec2(d,m);
}


vec3 normal(vec3 ray_hit_position, float smoothness, vec3 mouse_origin)
{   
    // From https://www.shadertoy.com/view/MdSGDW
    vec3 n;
    vec2 dn = vec2(smoothness, 0.0);
    n.x = gui(ray_hit_position + dn.xyy, mouse_origin).x - gui(ray_hit_position - dn.xyy, mouse_origin).x;
    n.y = gui(ray_hit_position + dn.yxy, mouse_origin).x - gui(ray_hit_position - dn.yxy, mouse_origin).x;
    n.z = gui(ray_hit_position + dn.yyx, mouse_origin).x - gui(ray_hit_position - dn.yyx, mouse_origin).x;
    return normalize(n);
}

vec2 raymarch(vec3 position, vec3 direction, vec3 mouse_origin )
{
    float total_distance = NEAR_CLIPPING_PLANE;
    for(int i = 0 ; i < NUMBER_OF_MARCH_STEPS ; ++i)
    {
        vec2 result = gui(position + direction * total_distance, mouse_origin);
        if(result.x < EPSILON)
        {
            return vec2(total_distance, result.y);
        }
        total_distance += result.x * DISTANCE_BIAS;
        if(total_distance > FAR_CLIPPING_PLANE) break;
    }
    return vec2(FAR_CLIPPING_PLANE, 0.0);
}


float renderWave(vec2 uv) {
    vec3 direction = normalize(vec3(uv,1.));
    vec3 camera_origin = vec3(uv.x,uv.y, -6.0);
    vec3 mouse_origin = camera_origin.z*vec3(0.0);
    vec2 result = raymarch(camera_origin, direction, mouse_origin);;
    float wave = max(min(result.x*0.19,1.0),.0)*2.0  - 1.0;
    return wave ;
}

vec4 renderShape(vec2 uv) {
 
    vec3 direction = normalize(vec3(uv,1.));
    vec3 camera_origin = vec3(uv.x,uv.y, -5);
    vec3 mouse_origin = camera_origin.z*vec3(0.0);

    vec2 result = raymarch(camera_origin, direction, mouse_origin);
    float fog = pow(1.0 / (1.0 + result.y), 0.5);
    vec3 intersection = camera_origin + direction * result.x;
    vec3 nrml = normal(intersection, 0.000001, mouse_origin);
    vec3 materialColor = vec3(0.0);
    
    if(result.y == 1.0)
    {
        float glow = 3.;
        vec3 light_dir = normalize(vec3(5,5,-1));
        float diffuse = dot(light_dir, nrml) * glow;
        diffuse = .1 + diffuse * 1.2;
        float diffuse2 = dot(normalize(mouse_origin), nrml) * glow;
        vec3 light_color = vec3(0.5);
        vec3 light2_color = vec3(.1,0,0);
        vec3 ambient_color = vec3(0.,0.,1.);
        materialColor = vec3(sin(result.x)+1.0) + 0.6* (diffuse * light_color + ambient_color);
    }
    
    vec3 diffuseLit = materialColor;
    return vec4(diffuseLit.rgb, 1.); // * fog; /* applying the fog last */
}


void main(void)
{
    vec2 uv = -1 + 2. * textureCoord;
    outColor = vec4(vec3(0.0),1.);
    
    // visualize 3d shape
    
    //vec2 shapeUV = uv * 1.0;
    //shapeUV.x += .2;
    //if (shapeUV.x < .2 && shapeUV.x > -.2) {
    //    vec4 cube = renderShape(shapeUV);
    //    if (cube.r > 0.001){
    //        outColor += cube *0.6;
    //    }
    //}
    
    // visualize wavetable shapes
    //vec2 waveUV = uv* 1.0;
    //waveUV.x -= .3;
    //if (waveUV.x < .2 && waveUV.x > -.2) {
    //float n = 0.0; // start WAVE plotting
    //for(int i = 0 ; i < NUM_WAVES ; ++i)
    //{
    //    float wave = renderWave(vec2(waveUV.x, n));
    //    if (abs((1.0-waveUV.y+n) - wave ) < 0.002) { 
    //        outColor.g += .8;
    //    }
    //    n += 0.04; // step size
    // }
    //}
    
    
   // gl_FragColor =  outColor;
}