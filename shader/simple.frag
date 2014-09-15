uniform vec4 color;
varying vec2 f_texcoord;

void main (void)
{
  vec2 texc = f_texcoord;
  gl_FragColor = vec4(0.000003*texc.x + color.x, color.y, color.z, 1.0);

  //gl_FragColor = vec4(0.3, 0.3, 0.4, 1.0);
  //gl_FragColor = vec4(red, 0.3, 0.4, 1.0);
  //vec4 ccolor = vec4(color.x, 0.3, 0.4, 1.0);
  //gl_FragColor = color;
}

