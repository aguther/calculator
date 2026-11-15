fn git(args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
}
fn main() {
    println!("cargo:rerun-if-env-changed=CALCULATOR_COMMIT");
    let mut paths = vec!["HEAD".to_owned(), "packed-refs".to_owned()];
    if let Some(reference) = git(&["symbolic-ref", "HEAD"]) {
        paths.push(reference);
    }
    for path in paths {
        if let Some(path) = git(&["rev-parse", "--git-path", &path])
            && std::path::Path::new(&path).exists()
        {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    let commit = std::env::var("CALCULATOR_COMMIT")
        .ok()
        .or_else(|| git(&["rev-parse", "--short=12", "HEAD"]))
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=CALCULATOR_COMMIT={commit}");
}
