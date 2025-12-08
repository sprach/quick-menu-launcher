use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Re-run if INI changes
    println!("cargo:rerun-if-changed=src/QikMenu.ini");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    
    // Source Path
    let src_path = Path::new(&manifest_dir).join("src").join("QikMenu.ini");
    
    // Target Dir Estimation
    // The PROFILE env var is 'debug' or 'release'.
    // We want to put the file in the workspace root's target/<profile> dir.
    // Assuming manifest_dir is .../Gitea/
    // We want .../Gitea/target/<profile>/
    
    let target_dir = Path::new(&manifest_dir).join("target").join(&profile);
    
    // Create target dir if it doesn't exist (it should, but just in case)
    let _ = fs::create_dir_all(&target_dir);

    let dest_path = target_dir.join("QikMenu.ini");

    if !dest_path.exists() {
        if let Err(e) = fs::copy(&src_path, &dest_path) {
            println!("cargo:warning=Failed to copy INI file: {}", e);
        } else {
            println!("cargo:warning=Copied INI file to {:?}", dest_path);
        }
    }

    // --- Icon Handling ---
    let icon_src = Path::new(&manifest_dir).join("assets").join("icon.jpg");
    let icon_dest = Path::new(&manifest_dir).join("assets").join("icon.ico");

    if icon_src.exists() && !icon_dest.exists() {
         // Convert JPG to ICO and PNG
         if let Ok(img) = image::open(&icon_src) {
             let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
             let _ = resized.save(&icon_dest);
             println!("cargo:warning=Converted and resized icon.jpg to icon.ico");
             
             // Generate small tray icon (64x64) for runtime loading
             let small = img.resize(64, 64, image::imageops::FilterType::Lanczos3);
             let small_dest = Path::new(&manifest_dir).join("assets").join("tray_icon.png");
             let _ = small.save(&small_dest);
         } else {
             println!("cargo:warning=Failed to load icon.jpg");
         }
    }

    if icon_dest.exists() {
        let mut res = winres::WindowsResource::new();
        res.set_icon(icon_dest.to_str().unwrap());
        if let Err(e) = res.compile() {
             println!("cargo:warning=Failed to compile resources: {}", e);
        }
    }
}
