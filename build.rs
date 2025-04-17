fn main() {
    println!("cargo:rerun-if-changed=.env");

    let _ = dotenvy::dotenv();

    if let Ok(api_key) = std::env::var("LASTFM_API_KEY") {
        println!("cargo:rustc-env=LASTFM_API_KEY={}", api_key);
    }
}
