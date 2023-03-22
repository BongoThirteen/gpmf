use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let dir = Path::new("/opt/git/gpmf-parser/samples");
    // let path=dir.join("hero5.mp4");
    let path = dir.join("max-360mode.mp4");
    let file = File::open(path)?;

    let size = file.metadata()?.len();
    let reader = BufReader::new(file);

    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    // Print boxes.
    println!("major brand: {}", mp4.ftyp.major_brand);
    println!("timescale: {}", mp4.moov.mvhd.timescale);

    // Use available methods.
    println!("size: {}", mp4.size());

    let mut compatible_brands = String::new();
    for brand in mp4.compatible_brands().iter() {
        compatible_brands.push_str(&brand.to_string());
        compatible_brands.push_str(",");
    }
    println!("compatible brands: {}", compatible_brands);
    println!("duration: {:?}", mp4.duration());

    // Track info.
    for (id, track) in mp4.tracks().iter() {
        println!("{} {:?}", id, track);
        // println!(
        //     "track: #{}({}) {} : {}",
        //     track.track_id(),
        //     track.language(),
        //     track.track_type()?,
        //     track.box_type()?,
        // );
    }
    Ok(())
}
