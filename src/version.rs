pub fn show() {
    println!("otree {}", env!("OTREE_VERSION"));

    println!();
    println!("commit: {}", env!("OTREE_SHA"));

    println!();
    println!("build type: {}", env!("OTREE_BUILD_TYPE"));
    println!("build target: {}", env!("OTREE_TARGET"));
    println!("build time: {}", env!("VERGEN_BUILD_TIMESTAMP"));

    println!();
    println!("rust version: {}", env!("VERGEN_RUSTC_SEMVER"));
    println!("rust channel: {}", env!("VERGEN_RUSTC_CHANNEL"));
    println!("llvm version: {}", env!("VERGEN_RUSTC_LLVM_VERSION"));

    if env!("VERGEN_CARGO_DEBUG") == "true" {
        println!();
        println!("[WARNING: Running in debug mode, the speed may be slow]");
    }
}
