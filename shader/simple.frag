precision highp float;
uniform vec4 color;
uniform sampler2D texture;
varying vec2 f_texcoord;

void main (void)
{
  //vec2 texc = f_texcoord;
  vec2 texc = vec2(f_texcoord.x, 1.0 - f_texcoord.y); //TODO fix texture reading
  vec4 diffuse_tex = texture2D(texture, texc);
  if (color.x < -1.5)
  {
  gl_FragColor = vec4(0.000003*texc.x + color.x, color.y, color.z, 1.0);
  }
  else {
  gl_FragColor = diffuse_tex;
  }

  //gl_FragColor = vec4(0.3, 0.3, 0.4, 1.0);
  //gl_FragColor = vec4(red, 0.3, 0.4, 1.0);
  //vec4 ccolor = vec4(color.x, 0.3, 0.4, 1.0);
  //gl_FragColor = color;
}

