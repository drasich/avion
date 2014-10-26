uniform sampler2D texture;
varying vec2 f_texcoord;

void main (void)
{
  vec2 texc = f_texcoord; 
  vec4 diffuse_tex = texture2D(texture, texc);
  if (diffuse_tex.x == 1.0)
  {
   gl_FragColor = vec4(0.0,0.0,0.0,0.0);
  }
  else {
    //gl_FragColor = diffuse_tex;
   gl_FragColor = vec4(1.0,0.0,0.0,1.0);
  }
}

