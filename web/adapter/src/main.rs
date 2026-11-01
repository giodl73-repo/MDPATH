use std::{env, fs::File, io::Read};
fn main() {
    let arg = env::args()
        .nth(1)
        .expect("Usage: mdpath-inspect REQUEST.json | --examples");
    if arg == "--examples" {
        println!("{}", mdpath_web::examples());
        return;
    }
    let mut text = String::new();
    File::open(arg)
        .expect("open request")
        .take((mdpath_web::MAX_REQUEST + 1) as u64)
        .read_to_string(&mut text)
        .expect("read UTF-8 request");
    println!("{}", mdpath_web::inspect(&text));
}
