layout(location=0) in vec4 position;
uniform vec2 u_mouse_delta;

void main() {
    gl_Position = position;
    gl_Position.x *= cos(u_mouse_delta.x);
    gl_Position.y *= cos(u_mouse_delta.y);
}