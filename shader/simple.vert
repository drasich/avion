attribute vec2 position;
void main(void)
{
  //gl_Position = matrix * vec4(vertex, 1.0);
  //gl_Position = vec4(0, 0, 0, 1.0);
  gl_Position = vec4(position, 0.0, 1.0);
}

