//! Reads `icon.png` out of real built `.smod` packages the way the game does (at the package
//! root, a square PNG of at most 512x512 and 256 KiB) and scales it to the list's 128x128.

use std::io::Read;
use std::path::{Path, PathBuf};

/// The icon of the `.smod` at `path`, decoded and scaled to 128x128 RGBA.
fn smod_icon(path: &Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entry = zip.by_name("icon.png").map_err(|e| format!("no icon.png: {e}"))?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > 256 * 1024 {
        return Err("icon.png over 256 KiB".into());
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("too large")?];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let (w, h) = (info.width as usize, info.height as usize);
    if w != h || w == 0 || w > 512 {
        return Err(format!("icon.png is {w}x{h}"));
    }
    if info.bit_depth != png::BitDepth::Eight {
        return Err("icon.png isn't 8 bits a channel".into());
    }
    let channels = info.color_type.samples();
    let mut out = Vec::with_capacity(128 * 128 * 4);
    for y in 0..128 {
        let sy = (2 * y + 1) * h / 256;
        for x in 0..128 {
            let sx = (2 * x + 1) * w / 256;
            let at = (sy * w + sx) * channels;
            let px = &buf[at..at + channels];
            match channels {
                4 => out.extend_from_slice(px),
                2 => out.extend_from_slice(&[px[0], px[0], px[0], px[1]]),
                _ => return Err(format!("{channels} channels")),
            }
        }
    }
    Ok(out)
}

/// This mod's own built package and the other mods' packages kept beside the projects.
fn built_smods() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found: Vec<PathBuf> = vec![root.join("target/sunrise-mod/sunnymods.smod")];
    if let Ok(dirs) = std::fs::read_dir(root.join("../dist")) {
        for dir in dirs.flatten() {
            let Ok(files) = std::fs::read_dir(dir.path()) else { continue };
            found.extend(
                files
                    .flatten()
                    .map(|f| f.path())
                    .filter(|p| p.extension().is_some_and(|e| e == "smod")),
            );
        }
    }
    found.retain(|p| p.is_file());
    found.sort();
    found
}

#[test]
fn icons_load_from_real_built_packages() {
    let smods = built_smods();
    if smods.is_empty() {
        eprintln!("no built .smod here; build with `cargo sunrise-mod build` first");
        return;
    }
    let mut loaded = 0;
    for path in &smods {
        match smod_icon(path) {
            Ok(rgba) => {
                assert_eq!(rgba.len(), 128 * 128 * 4, "{}", path.display());
                let seen = rgba.chunks(4).filter(|p| p[3] > 0).count();
                assert!(seen > 128, "{}: icon is nearly empty", path.display());
                loaded += 1;
            }
            Err(e) if e.starts_with("no icon.png") => {}
            Err(e) => panic!("{}: {e}", path.display()),
        }
    }
    assert!(loaded > 0, "none of {} packages had an icon", smods.len());
    eprintln!("icons loaded from {loaded} of {} packages", smods.len());
}
