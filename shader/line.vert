attribute vec3 position;
attribute vec4 color;
uniform mat4 matrix;

varying vec4 vcolor;

void main(void)
{
  vcolor = color;
  gl_Position = matrix * vec4(position, 1.0);
}

