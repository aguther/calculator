//! Embed the source revision and Windows shell icon during compilation.
fn git_output(arguments: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(arguments)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
}
fn main() {
    println!("cargo:rerun-if-changed=../../assets/windows/calculator.ico");
    // Resource compilation needs the Windows SDK only for Windows targets.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("../../assets/windows/calculator.ico")
            .compile()
            .expect("compile the Windows application icon resource");
    }

    println!("cargo:rerun-if-env-changed=CALCULATOR_COMMIT");
    let mut paths = vec!["HEAD".to_owned(), "packed-refs".to_owned()];
    if let Some(reference) = git_output(&["symbolic-ref", "HEAD"]) {
        paths.push(reference);
    }
    for path in paths {
        if let Some(path) = git_output(&["rev-parse", "--git-path", &path])
            && std::path::Path::new(&path).exists()
        {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    let commit = std::env::var("CALCULATOR_COMMIT")
        .ok()
        .or_else(|| git_output(&["rev-parse", "--short=12", "HEAD"]))
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=CALCULATOR_COMMIT={commit}");
}
