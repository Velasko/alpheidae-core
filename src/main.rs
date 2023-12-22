mod core;
mod errors;
mod traits;

fn print<T>(a: T, b: T) {
    println!("a")
}

fn main() {
    // let engine = crate::core::engine::Engine::new();
    let vec = (0..20).map(|x| [x, x + 1]).collect::<Vec<[i32; 2]>>();
    let pool = core::thread_pool::Pool::default();
    let ret = pool.star_map(print, vec.clone());
    println!("ret: {:?}", ret);
}
