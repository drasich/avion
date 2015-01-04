uniform vec4 color;

void main (void)
{
  vec4  yep = color;
  yep.w = 1.0;
  gl_FragColor = yep;

  //gl_FragColor = color;
}

