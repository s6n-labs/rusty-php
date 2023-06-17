fn main() {
    println!("cargo:rustc-link-search=/usr/local/lib");
    println!("cargo:rustc-link-search=/usr/lib");
    println!("cargo:rustc-link-lib=dylib=php");
    // println!("cargo:rustc-link-lib=dylib=ssl");
    // println!("cargo:rustc-link-lib=dylib=crypto");
    // println!("cargo:rustc-link-lib=dylib=readline");
    // println!("cargo:rustc-link-lib=dylib=argon2");
    // println!("cargo:rustc-link-lib=dylib=curl");
    // println!("cargo:rustc-link-lib=dylib=onig");
    // println!("cargo:rustc-link-lib=dylib=z");
}
