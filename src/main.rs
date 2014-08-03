#[link(name = "joker")]
extern {
    fn simple_window_init();
    fn elm_simple_window_main();
}

mod shader;
mod mesh;

fn main() {
    unsafe {
        simple_window_init();
    };

    spawn(proc() {
        shader::shader_init();
    });

    spawn(proc() {
        mesh::mesh_init();
    });

    unsafe { 
        elm_simple_window_main();
    };
}

