#ifdef GL_ES
precision highp float;
#endif

uniform float time;
uniform vec2 resolution;
uniform vec2 mouse;
uniform sampler2D midi1;
uniform vec3 spectrum0;
uniform sampler2D spectrum1;

uniform sampler2D prevFrame;
uniform sampler2D prevPass;

uniform sampler2D tex1;
uniform sampler2D tex2;
uniform sampler2D tex3;

uniform sampler2D noise1;

varying vec3 v_normal;
varying vec2 v_texcoord;

#define NEAR_CLIPPING_PLANE 1.0
#define FAR_CLIPPING_PLANE 50.0
#define NUMBER_OF_MARCH_STEPS 50
#define EPSILON 0.001
#define DISTANCE_BIAS 1
#define PI 3.14159
#define PI2 2*PI

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


float sdTunnel(vec3 p, float s)
{
	return length(p.xy+vec2(s*sin(p.z+time),s*cos(p.z+time)))-s;
}


float sdTorus( vec3 p, vec2 t )
{
  vec2 q = vec2(length(p.xz)-t.x,p.y);
  return length(q)-t.y;
}

float sdSphere(vec3 p, float s)
{
	return length(p) - s;
}



float sdCylinder( vec3 p, vec3 c )
{

  return length( p.xz - c.xy ) - c.z;
}

float sdBox( vec3 p, vec3 b )
{
  vec3 d = abs(p) - b;
  return min(max(d.x,max(d.y,d.z)),0.0) +
         length(max(d,0.0));
}

float unionSDF(float distA, float distB) {
    return min(distA, distB);
}

float differenceSDF(float distA, float distB) {
    return max(distA, -distB);
}

float intersectSDF(float distA, float distB) {
    return max(distA, distB);
}

float sminSDF(float a, float b, float k = 32)
{
    float res = exp(-k*a) + exp(-k*b);
    return -log(max(0.0001,res)) / k;
}

vec2 gui(in vec3 pos, vec3 mouse3d = vec3(0)) {

    vec3 p = rotateY(rotateX(pos+vec3(0,-1,0),mouse3d.y), mouse3d.x);
    float d1 = sdBox(p,vec3(1.));

 

    float m = 1.;



    return vec2(d1,m);
}

vec3 normal(vec3 ray_hit_position, float smoothness, vec3 mouse_origin)
{	
    // From https://www.shadertoy.com/view/MdSGDW
	vec3 n;
	vec2 dn = vec2(smoothness, 0.0);
	n.x	= gui(ray_hit_position + dn.xyy, mouse_origin).x - gui(ray_hit_position - dn.xyy, mouse_origin).x;
	n.y	= gui(ray_hit_position + dn.yxy, mouse_origin).x - gui(ray_hit_position - dn.yxy, mouse_origin).x;
	n.z	= gui(ray_hit_position + dn.yyx, mouse_origin).x - gui(ray_hit_position - dn.yyx, mouse_origin).x;
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



vec4 render(vec2 uv = v_texcoord * 2. - 2.) {

 
    vec3 direction = normalize(vec3(uv,1.));
    vec3 camera_origin = vec3(0.,0., -5);
    vec3 mouse_origin = camera_origin.z*vec3(1.-mouse*2,.15);

    vec2 result = raymarch(camera_origin, direction, mouse_origin);
    float fog = pow(1.0 / (1.0 + result.y), 0.5);
    vec3 intersection = camera_origin + direction * result.x;
    vec3 nrml = normal(intersection, 0.000001, mouse_origin);
    vec3 materialColor = vec3(0);
    if(result.y == 1.0)
    {
        float glow = 10.;
        vec3 light_dir = normalize(vec3(1,5,-1));
   	    float diffuse = dot(light_dir, nrml) * glow;
        diffuse = .1 + diffuse * 0.9;
        float diffuse2 = dot(normalize(mouse_origin), nrml) * glow;
        vec3 light_color = vec3(0.5);
        vec3 light2_color = vec3(.1,0,0);
        vec3 ambient_color = vec3(0,1.,.1);
        materialColor = vec3(sin(result.x)+2.0) + 0.5* (diffuse * light_color + ambient_color);
    }
    if (uv.x > 0.7 && uv.x < .8) {

        return vec4(0.,1.,0.,1.);
    }

    vec3 diffuseLit = materialColor ;
	return vec4(diffuseLit.rgb, 1.) * fog; /* applying the fog last */
}

void main(void)
{
    vec2 uv = v_texcoord;

    gl_FragColor=render();
}