extern crate clap;
extern crate temporary;
extern crate unrar;
extern crate walkdir;
extern crate zip;

use clap::{App, Arg};

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let matches = App::new("rar2zip")
        .version("0.1")
        .author("Keringar <git@keringar.xyz>")
        .about("Converts rar files into zip files")
        .arg(
            Arg::with_name("INPUTS")
                .help("Sets the input file to use, accepts multiple arguments separated by spaces")
                .multiple(true)
                .required(true),
        )
        .get_matches();

    let path_args = matches.values_of("INPUTS").unwrap();

    for path in path_args {
        let rar = unrar::Archive::new(path.to_string());

        if !rar.is_archive() {
            eprintln!("ERROR: Provided file is not a rar archive");
            return;
        }

        let tmp_dir = temporary::Directory::new("extract").unwrap();

        match rar.extract_to(String::from(tmp_dir.to_str().unwrap())) {
            Ok(mut open_archive) => if let Err(_) = open_archive.process() {
                eprintln!("ERROR: Could not extract rar archive");
                return;
            },
            Err(_) => {
                eprintln!("ERROR: Could not open rar archive");
                return;
            }
        }

        let extracted_name = Path::new(path).file_stem().unwrap();

        let source_dir = tmp_dir.join(extracted_name);

        let out_path = Path::new(extracted_name).with_extension("zip");
        let out_file = File::create(&out_path).unwrap();

        let mut zip = zip::ZipWriter::new(&out_file);

        let options = zip::write::FileOptions::default();

        let source_walk = walkdir::WalkDir::new(&source_dir);

        for dent in source_walk.into_iter().filter_map(|e| e.ok()) {
            let path = dent.path();
            let name = path.strip_prefix(Path::new(&source_dir))
                .unwrap()
                .to_str()
                .unwrap();

            if path.is_file() {
                println!("Adding {:?} as {:?} ...", path, name);
                zip.start_file(name, options).unwrap();
                let mut f = File::open(path).unwrap();
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&*buffer).unwrap();
            }
        }

        zip.finish().unwrap();
    }
}
