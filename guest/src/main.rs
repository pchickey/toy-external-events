#[wstd::main]
async fn main() {
    println!("Hello, world!");
    wstd::task::sleep(wstd::time::Duration::from_micros(10)).await;
    println!("That was a nice nap");
}
