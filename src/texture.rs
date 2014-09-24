use png;
use libc::{c_uint, c_void};
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::mem;

pub struct CglTexture;

#[link(name = "cypher")]
extern {
    pub fn cgl_texture_init(
        data : *const c_void,
        internal_format : c_uint,
        width : c_uint,
        height : c_uint
        ) -> *const CglTexture;
}

pub struct Texture
{
    name : String,
    pub state : i32,
    pub image : Option<png::Image>,
    pub cgl_texture: Option<*const CglTexture>,
} 
impl Texture
{
    pub fn new(name :&str) -> Texture
    {
        let mut t = Texture{
            name: String::from_str(name),
            state : 0,
            image : None,
            cgl_texture : None
        };

        t.load();

        t
    }

    pub fn load(&mut self)
    {
        if self.state != 0 {
            return
        }

        let result = png::load_png(&Path::new(self.name.as_slice()));
        
        match (result) {
            Err(e) => {},
            Ok(img) => {
                self.image = Some(img);
                self.state = 1;
            }
        };
    }

    pub fn init(&mut self)
    {
        if self.state != 1 {
            return
        }

        match self.image  {
            None => {},
            Some(ref img) => {
                unsafe {
                    let cgltex =  cgl_texture_init(
                        mem::transmute(img.pixels.as_ptr()),
                        1,
                        img.width as c_uint,
                        img.height as c_uint
                        );

                    self.cgl_texture = Some(cgltex);
                }
                self.state = 2;
            }
        }
    }
}

impl <S: Encoder<E>, E> Encodable<S, E> for Texture {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Texture", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Texture {
  fn decode(decoder: &mut S) -> Result<Texture, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Texture{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
           state : 0,
           image : None,
           cgl_texture : None
        })
    })
  }
}


