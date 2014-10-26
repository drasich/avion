uniform sampler2D texture;
uniform sampler2D texture_all;
varying vec2 f_texcoord;

void main (void)
{
  //vec4 allz = texture2D(texture_all, vec2(gl_FragCoord.x/resolution.x,gl_FragCoord.y/resolution.y));

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

