fn main() {
    // Recompile le crate quand une migration est ajoutée ou modifiée.
    println!("cargo:rerun-if-changed=../../migrations");
}
