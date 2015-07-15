use png;
use libc::{c_uint, c_void};
use rustc_serialize::{Encodable, Encoder, Decoder, Decodable};
use std::mem;
use std::path::Path;
use std::cell::Cell;

#[repr(C)]
pub struct CglTexture;

unsafe impl Send for CglTexture {}
unsafe impl Sync for CglTexture {}

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
    //state : i32,
    image : Option<png::Image>,
    pub cgl_texture: Option<*const CglTexture>,
} 

unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

impl Texture
{
    pub fn new(name :&str) -> Texture
    {
        let t = Texture{
            name: String::from(name),
            state : 0,
            image : None,
            cgl_texture : None
        };

        t
    }

    pub fn load(&mut self)
    {
        if self.state != 0 {
            return
        }

        //let result = png::load_png(&Path::new(self.name.as_str()));
        let path : &Path = self.name.as_ref();
        let result = png::load_png(path);
        
        match result {
            Err(_) => {},
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
                //*
                 let data = match img.pixels {
                     //png::RGB8(ref pixels) => pixels.as_ptr(),
                     png::PixelsByColorType::RGBA8(ref pixels) =>  { 
                         for i in 0usize..8 { 
                             println!("{}RGBA{} : {}", self.name, i, pixels[i]);
                         }
                         pixels.as_ptr()
                     },
                     //png::K8(ref pixels) => pixels.as_ptr(),
                     //png::KA8(ref pixels) => pixels.as_ptr(),
                     _ => { println!("it's not rgba8"); return; }
                 };
                 //*/

                unsafe {

                    let cgltex =  cgl_texture_init(
                        //mem::transmute(img.pixels.as_ptr()),
                        mem::transmute(data),
                        4,
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

impl Encodable for Texture {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Texture", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Texture {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Texture, D::Error> {
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


