use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let skill_src = Path::new(&manifest_dir).join("skill").join("ccc");
    let skm_path = skill_src.join("SKILL.md");

    let out_bin = Path::new(&manifest_dir).join("src").join("ccc_skill.bin");

    if !skm_path.exists() {
        println!("cargo:warning=skill/ccc/SKILL.md not found, creating empty skill");
        // Write a valid zip with empty SKILL.md
        let file = File::create(&out_bin).expect("Failed to create skill bin");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("SKILL.md", options).expect("Failed to start zip entry");
        zip.write_all(b"# Skill not available\n").expect("Failed to write");
        zip.finish().expect("Failed to finish zip");
        println!("cargo:warning=Empty skill bin created at {}", out_bin.display());
        println!("cargo:rerun-if-changed={}", skill_src.display());
        return;
    }

    // Create zip
    let file = File::create(&out_bin).expect("Failed to create skill bin");
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let skm_content = fs::read(&skm_path).expect("Failed to read SKILL.md");
    zip.start_file("SKILL.md", options).expect("Failed to start zip entry");
    zip.write_all(&skm_content).expect("Failed to write SKILL.md to zip");
    zip.finish().expect("Failed to finish zip");

    println!("cargo:warning=Skill bin created at {}", out_bin.display());

    // Tell Cargo to rerun if the skill file changes
    println!("cargo:rerun-if-changed={}", skm_path.display());
}
