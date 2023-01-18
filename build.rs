// Adapted from: https://codecrash.me/an-opengl-preprocessor-for-rust

use eyre::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tera::*;

const OUTUT_PATH: &'static str = "shaders_stitched";

fn generate_shaders() -> Result<()> {
    println!("cargo:rerun-if-changed=shaders/");

    let tera = Tera::new("shaders/**")?;
    let context = Context::new();

    fs::create_dir_all(OUTUT_PATH)?;

    visit_dirs(&PathBuf::from("shaders/"), &tera, &context)?;

    Ok(())
}

const NOTICE: &'static str =
    "// !!!\n// THIS FILE WAS AUTOGENERATED, MODIFICATIONS WILL HAVE NO EFFECT\n// !!!\n";

fn visit_dirs(dir: &Path, tera: &Tera, context: &Context) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, tera, context)?;
            } else {
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap();
                let path = path.strip_prefix("shaders/")?.to_str().unwrap();

                if !file_name.ends_with("glsl") {
                    let mut result = Vec::from(NOTICE);
                    tera.render_to(path, &context, &mut result)?;

                    let output_path = format!("{}/{}", OUTUT_PATH, file_name);
                    fs::write(output_path, result)?;
                }
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = generate_shaders() {
        // panic here for a nicer error message, otherwise it will
        // be flattened to one line for some reason
        panic!("Unable to generate shaders\n{}", err);
    }
}
