precision mediump float;
out vec4 out_color;
uniform vec2 u_mouse_delta;
uniform uint u_zoom;

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec2 wedge(vec2 a, vec2 b) {
    return vec2(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}

void main () {
    vec2 c = -1.0 + 2.0 * gl_FragCoord.xy / 1000;
    //scale factor
    c *= 1/float(u_zoom);

    //offset
    vec2 mouse_delta = u_mouse_delta*1/float(u_zoom);
    c += vec2(-1-mouse_delta.x, mouse_delta.y);
    vec2 z = c;

    float iterations = 0;
    for (int i=0; i<1000;i++) {
        z = wedge(z,z) + c;
        if (length(z) > 4) {
            break;
        }
        iterations += 0.01;
    }

    if (length(z) < 2) {
        out_color = vec4(0.0,0.0,0.0,1.0); //black
    } else {
        out_color = vec4(hsv2rgb(vec3(iterations, 1.0, 1.0)), 1.0);
    }
}