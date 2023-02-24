/*
 * JLOF
 *
 */

// https://github.com/marklundin/glsl-sdf-primitives
// https://github.com/marklundin/glsl-sdf-ops
// https://github.com/glslify/glslify

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

// these constants are used throughout the shader, 
// they can be altered to avoid glitches or optimize the framerate,
// their meaning can best be seen in context below
#define NEAR_CLIPPING_PLANE 3
#define FAR_CLIPPING_PLANE 500.0
#define NUMBER_OF_MARCH_STEPS 250
#define EPSILON 0.001
#define DISTANCE_BIAS 0.9
#define PI 3.14159
#define PI2 2*PI

vec4 pf(vec2 uv = v_texcoord) {
    return texture2D(prevFrame,vec2(uv.x,1-uv.y));
}

vec2 midiCoord(float offset)
{
    return vec2(mod(offset,32.),offset / 32.)/32.;
}

// midi controller 
float cc(int c, sampler2D m = midi1) {
 float ccc = 3. * 127. + float(c);
 return texture2D(m,midiCoord(ccc)).w;
}

vec4 tex(vec2 c, sampler2D t = tex1) {
    return texture2D(t, c.xy); 
}

vec4 analyzer(float x) {
    float n = 16;
    vec2 s = vec2(mod(x*n,1),mod(x,1));
    vec4 c = texture2D(spectrum1,s)*10;
    return c;
}

vec2 plasma(
    vec2 uv = v_texcoord, 
    float time = time * .1, 
    vec4 m = vec4(1.+sin(time*.1)*2.,3.,2.,.1)
    ) {
    return vec2(
        abs(sin(cos(time+m.x*uv.y)*m.z*uv.x+time*m.a)),
        abs(cos(sin(time+m.x*uv.x)*m.z*uv.y+time*m.a))
    );
}

#define Rotate(p,a,r)mix(a*dot(p,a),p,cos(r))+sin(r)*cross(p,a)
#define H(h)cos(h*6.3+vec3(0,23,21))*.5+.5
vec4 f1(
    vec2 uv = v_texcoord,
    float ray = 2.0,
    float rotationSpeed = time*.1,
    float mdf = .01
) {
    vec4 O=vec4(0);
    vec3 r=vec3(1);
    vec3 p;
    float g=0.,e=0.,s=0.;
    for(float i=0.;i<=99.;++i)
    {
        p=g*vec3(uv-.5,1);
        p.z-=1.;
        p=Rotate(p,normalize(vec3(0,1,i*.04)),rotationSpeed);
        s=3.;
        s*=e=1./min(dot(p,p),1.);
        p=abs(p)*e-mdf;
        g+=e=length(p.xy)/s;
        if (e < .1 * abs(2. * cos(ray)) ) {
            O.xyz += 1./i;
        }
 
    }
    return O;
}

float fmod(float a, float b)
{
    if(a<0.0)
    {
        return b - mod(abs(a), b);
    }
    return mod(a, b);
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
   
float sdTorus( vec3 p, vec2 t )
{
  vec2 q = vec2(length(p.xz)-t.x,p.y);
  return length(q)-t.y;
}

float sdCapsule( vec3 p, vec3 a, vec3 b, float r )
{
    vec3 pa = p - a, ba = b - a;
    float h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
    return length( pa - ba*h ) - r;
}

float sdCone( vec3 p, vec2 c, float h )
{
  float q = length(p.xz);
  return max(dot(c.xy,vec2(q,p.y)),-h-p.y);
}

float sdCutHollowSphere( vec3 p, float r, float h, float t )
{
  // sampling independent computations (only depend on shape)
  float w = sqrt(r*r-h*h);
  
  // sampling dependant computations
  vec2 q = vec2( length(p.xz), p.y );
  return ((h*q.x<w*q.y) ? length(q-vec2(w,h)) : 
                          abs(length(q)-r) ) - t;
}

float sdCutSphere( in vec3 p, in float r, in float h )
{
    float w = sqrt(r*r-h*h); // constant for a given shape
    vec2 q = vec2( length(p.xz), p.y );
    float s = max( (h-r)*q.x*q.x+w*w*(h+r-2.0*q.y), h*q.x-w*q.y );
    return (s<0.0) ? length(q)-r :
           (q.x<w) ? h - q.y     :
                     length(q-vec2(w,h));
}

float intersectSDF(float distA, float distB) {
    return max(distA, distB);
}

float unionSDF(float distA, float distB) {
    return min(distA, distB);
}

float differenceSDF(float distA, float distB) {
    return max(distA, -distB);
}

float blendSDF(float d1, float d2, float a) {
    return a * d1 + (1 - a) * d2;
}

float sminSDF(float a, float b, float k = 32)
{
    float res = exp(-k*a) + exp(-k*b);
    return -log(max(0.0001,res)) / k;
}

float fOpPipe(float a, float b, float r) {
	return length(vec2(a, b)) - r;
}

float sdEllipsoid(vec3 pos,vec3 cen, vec3 rad)
{
    vec3 p = pos - cen;
    float k0 = length(p/rad);
    float k1 = length(p/(rad*rad));
    return k0*(k0-1.0)/k1;
}

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


float tail(vec3 pos, float iTime = time)
{
    float t = -.9;
    float d = pos.z > t ? 0 : pos.z-t;
    vec3 p = vec3(pos.x,pos.y-.3+d*(sin(d+iTime)),pos.z);
    return sdEllipsoid(p, vec3(0,0,-1.9), vec3(.2,.4,1));
}

float frontlimb(vec3 p, float a = -1, float iTime = time *2) {
    return sdCapsule(p+vec3(a*.3,1,0), vec3(1*-a,.2,a*sin(iTime)), vec3(-a*.3,0,0), .2);
}

float backlimb(vec3 p, float a = -1) {
    return sdCapsule(p+vec3(-a*.2,-.5,0), vec3(a*.5,.7,a*sin(time*2)), vec3(-a*.2,0,0), .2);
}

vec2 cat(in vec3 pos) {
    vec3 axis = normalize(vec3(0,1,.6));

    vec3 p = rotateY(pos+vec3(0,-1,0), 0);
    p = rotateY(p, time*0.1);
    p = rotateZ(p, sin(time)*0.3);
    
    // array of cats

    vec3 c = vec3(5,0,6);
    p = mod(p+0.5*c,c)-0.5*c;

    //material
    float m = 1.0;

    float body = sminSDF(sdSphere(p+vec3(0,1,0), .5), sdSphere(p, 1), 3);
    body = sminSDF(body,tail(p),7);
    
    vec3 phead = p+vec3(0.1*sin(time),2,.2);
    float head = sdSphere(phead, .8);

    //ears
    float ear1 = sdCutSphere(rotateX(phead+vec3(.5,1,0),80), .3, .1);
    float ear2 = sdCutSphere(rotateX(phead+vec3(-.5,1,0),80), .3, .1);
    float ears = unionSDF(ear1,ear2);
    head = sminSDF(head,ears,6);

    //nose
    float nose1 = sdEllipsoid(phead, vec3(0,0,.5), vec3(.5,.2,.5));
    float nose2 = sdEllipsoid(phead, vec3(0,-.1,.5), vec3(.2,.1,.5));
    float nose = unionSDF(nose1,nose2);
    
    head = sminSDF(head,nose);

    // eyes
    vec3 peye1 = phead+vec3(.4,.2,-.5);
    vec3 peye2 = phead+vec3(-.4,.2,-.5);
    float eye1a = sdSphere(peye1, .2);
    float eye1b = sdSphere(peye1-vec3(-.1,0.06*sin(time),.1), .1);
    float eye1 = unionSDF(eye1a,eye1b);
    float eye2a = sdSphere(peye2, .2);
    float eye2b = sdSphere(peye2-vec3(.1,0.06*sin(time),.1), .1);
    float eye2 = unionSDF(eye2a,eye2b);
    float eyes = unionSDF(eye1,eye2);
    if (eyes < head) { m = 3.0; }
    if (eye1b < head) { m = 2.0; }
    if (eye2b < head) { m = 2.0; }
    head = unionSDF(head, eyes);

    // collar
    float collar = fOpPipe(body, head, .1);

    //limbs
    float leg1 = frontlimb(p,1);
    float leg2 = frontlimb(p,-1);
    float flegs = unionSDF(leg1, leg2);
    float leg3 = backlimb(p,1);
    float leg4 = backlimb(p,-1);
    float blegs = unionSDF(leg3, leg4);

    body = sminSDF(body,blegs, 3);
    body = sminSDF(body,flegs, 10);

    float cat = unionSDF(head,body);

    if(collar < cat) { m = 4.0; }
    cat = unionSDF(cat,collar);
    return vec2(cat, m);
}

vec3 normal(vec3 ray_hit_position, float smoothness)
{	
    // From https://www.shadertoy.com/view/MdSGDW
	vec3 n;
	vec2 dn = vec2(smoothness, 0.0);
	n.x	= cat(ray_hit_position + dn.xyy).x - cat(ray_hit_position - dn.xyy).x;
	n.y	= cat(ray_hit_position + dn.yxy).x - cat(ray_hit_position - dn.yxy).x;
	n.z	= cat(ray_hit_position + dn.yyx).x - cat(ray_hit_position - dn.yyx).x;
	return normalize(n);
}

vec2 raymarch(vec3 position, vec3 direction)
{
    float total_distance = NEAR_CLIPPING_PLANE;
    for(int i = 0 ; i < NUMBER_OF_MARCH_STEPS ; ++i)
    {
        vec2 result = cat(position + direction * total_distance);
        if(result.x < EPSILON)
        {
            return vec2(total_distance, result.y);
        }
        total_distance += result.x * DISTANCE_BIAS;
        if(total_distance > FAR_CLIPPING_PLANE) break;
    }
    return vec2(FAR_CLIPPING_PLANE, 0.0);
}

vec4 render(vec2 uv = v_texcoord * 2 - 1)
{

    vec3 direction = normalize(vec3(uv, 1.5));
    vec3 camera_origin = vec3(0, 0, -10); // you can move the camera here
    vec2 result = raymarch(camera_origin, direction); // this raymarches the scene
    
    // arbitrary fog to hide artifacts near the far plane
    // 1.0 / distance results in a nice fog that starts white
    // but if distance is 0 
    float fog = pow(1.0 / (1.0 + result.y), 0.5);
    vec3 intersection = camera_origin + direction * result.x;
    vec3 nrml = normal(intersection, 0.001);

    // now let's pick a color
    vec3 materialColor = vec3(0);
    float glow = 1;
    if(result.y == 1.0)
    {
        materialColor = vec3(.7,.4,.3*nrml.g);
    }
    if(result.y == 2.0)
    {
       	materialColor = vec3(0,0,.5);
        glow = 20;
    }
    if(result.y == 3.0) 
    {
        materialColor = vec3(1.);
    }
    if(result.y == 4.0) 
    {
        materialColor = vec3(tex(nrml.xy*0.2,tex1));
        glow = 10;
    }
    if(result.y == 5.0) {
        materialColor = vec3(1,.5,0);
    }

    
    // We can reconstruct the intersection point using the distance and original ray

    
    // The normals can be retrieved in a fast way
    // by taking samples close to the end-result sample
    // their resulting distances to the world are used to see how the surface curves in 3D
    // This math I always steal from somewhere ;)
 
    
    // Lambert lighting is the dot product of a directional light and the normal
    vec3 light_dir = normalize(vec3(0,6,-8));
   	float diffuse = dot(light_dir, nrml) * glow;
    // Wrap the lighting around
    // https://developer.valvesoftware.com/wiki/Half_Lambert
    diffuse = .1 + diffuse * 0.9;
    // For real diffuse, use this instead (to avoid negative light)
    // diffuse = max(0.0, diffuse);
    
    // Combine ambient light and diffuse lit directional light
    vec3 light_color = vec3(.5);
    vec3 ambient_color = vec3(0,0,.4);
    vec3 diffuseLit = materialColor * (diffuse * light_color + ambient_color);
	return vec4(diffuseLit, result.x >= FAR_CLIPPING_PLANE*.5 ? 0 : 1) * fog; /* applying the fog last */
}

void main(void)
{
    vec2 uv = v_texcoord;
    float pxy = 1/resolution.y;
    float s = .2;
    float songPos = cc(1);
    float patPos = cc(2);
    float bd = cc(3);
    float sd = cc(4);
    float hh = cc(5);
    float bass = cc(6);
    float acid = cc(7);
    float glizes = cc(8);
    float arps = cc(9);
    float atmos = cc(10);

    float xad = 1.-abs(uv.x-.5)+sin(time*0.1);
    float yad = 1.-abs(uv.y-.5);
    float syad = sin(yad);
    vec2 p2 = abs(plasma(vec2(xad,xad))*2.0*sin(time*0.01));
    vec2 p1 = abs(plasma(vec2(xad,uv.y))*0.6*sin(time*sd/bd*0.1));

    vec4 l0 = pf(uv);
    vec4 l1 = vec4(0);

    if (bass > .1) {
        l1 += tex(abs(uv-sin(uv.y*uv.x+time)*vec2(uv.y))*bass*abs(sin(time*0.1)),tex1);
    l1 *= tex(abs(uv-sin(uv.y*1.-uv.x+time)*vec2(uv.y))*bass*abs(sin(time*0.1)),tex1);

    } 
    vec4 l2 = tex(p2+vec2(1.,sin(time*10.)),tex1)*bass ;
    vec4 l3 = vec4(glizes)*l2;    

    vec4 l4 = vec4(tex(vec2(sin(time)+xad*sd,yad*sd),tex3).rg,0,1);

    vec4 mix = l1;

    if (glizes > .2) {
        mix += l3;
    } 

    if (sd > .2) {
        mix +=l4;
    }

    if (bass > .4) {
        mix += l1*bd*bd + l2;
    }

    mix += render(uv);





    if (l3.a > 0) {
        mix = l3;
    } else {
        mix += l2;
    }




    gl_FragColor=mix;
}
