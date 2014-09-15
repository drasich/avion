attribute vec3 position;
attribute vec2 texcoord;
uniform mat4 matrix;

varying vec2 f_texcoord;

void main(void)
{
  gl_Position = matrix * vec4(position, 1.0);
  f_texcoord = texcoord;
}

