uniform vec4 color;

void main (void)
{
  //gl_FragColor = vec4(0.3, 0.3, 0.4, 1.0);
  //gl_FragColor = vec4(red, 0.3, 0.4, 1.0);
  //vec4 ccolor = vec4(color.x, 0.3, 0.4, 1.0);
  gl_FragColor = color;
}

