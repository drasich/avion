#[link(name = "slime")]
extern {
    fn elm_simple_window_main();
}

fn main() {
    println!("yo");

    spawn(proc() {
        unsafe { 
            elm_simple_window_main();
        };
    });

    println!("test");
}

