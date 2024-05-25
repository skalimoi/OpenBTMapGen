use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use log::{error, info, trace};

pub fn gdal_check() {
    if !Path::new("utils/gdal/").exists() | !Path::new("utils/gdal/gdal_translate.exe").exists() | !Path::new("utils/gdal/gdalwarp.exe").exists() {
        error!("GDAL directory doesn't exist or is unreachable! Has it been deleted?");
        error!("Soil creation will fail!")
    } else {
        info!("GDAL library checked at ./utils/gdal/\nOk!");
    }
    if std::env::var_os("PROJ_LIB").is_none() {
        std::env::set_var("PROJ_LIB", Path::new("utils/gdal/projlib").canonicalize().unwrap())
    }
    if std::env::var_os("GDAL_DATA").is_none() {
        std::env::set_var("GDAL_DATA", Path::new("utils/gdal/").canonicalize().unwrap())
    }
}

pub fn whitebox_check() {
    if !Path::new("utils/wbt/").exists() | !Path::new("utils/wbt/whitebox_tools.exe").exists() {
        error!("Whitebox Tools directory doesn't exist or is unreachable! Has it been deleted?");
        error!("Soil creation will fail!")
    } else {
        info!("Whitebox Open Core library checked at ./utils/wbt/\nOk!");
    }
}

pub fn geomorphons() {
    let output = Command::new(Path::new("utils/wbs/whitebox_tools.exe").canonicalize().unwrap())
        .args([
            "-r=Geomorphons",
            "-v",
            format!("--wd={}", Path::new("cache").canonicalize().unwrap().to_str().unwrap()).as_str(),
            "--dem=georref.tif",
            "-o=gm.tif",
            "--search=150",
            "--threshold=0.5",
            "--tdist=0.0",
            "--forms"
        ])
        .output().unwrap();
    info!("{}", from_utf8(&output.stdout).unwrap());
    trace!("{}", from_utf8(&output.stderr).unwrap());
}

pub fn heightpercentile() {
    let output = Command::new(Path::new("utils/wbs/whitebox_tools.exe").canonicalize().unwrap())
        .args([
            "-r=Geomorphons",
            "-v",
            format!("--wd={}", Path::new("cache").canonicalize().unwrap().to_str().unwrap()).as_str(),
            "--dem=georref.tif",
            "-o=gm.tif",
            "--search=150",
            "--threshold=0.5",
            "--tdist=0.0",
            "--forms"
        ])
        .output().unwrap();
    info!("{}", from_utf8(&output.stdout).unwrap());
    trace!("{}", from_utf8(&output.stderr).unwrap());
}

pub fn georreference() {
    let path = Path::new("utils/gdal/gdal_translate.exe").canonicalize().unwrap();
    let output = Command::new(path)
        .args([
            "-of",
            "GTiff",
            "-gcp",
            "0", 
                  "0",
                  "-5.483",
                  "9.637",
            "-gcp",
                  "8192",
                  "0",
                  "-4.917",
                  "9.637",
            "-gcp",
              "0",
              "8192",
                  "-5.483",
                  "9.071",
            "-gcp",
                  "8192",
                  "8192",
                  "-4.917",
                  "9.071",
            "D:\\Archivos\\OpenBTMapGen\\cache\\map.png",
            "D:\\Archivos\\OpenBTMapGen\\cache\\map.tif"
        ])
        .output().expect("Error running command!");
    info!("{}", from_utf8(&output.stdout).unwrap());
    trace!("{}", from_utf8(&output.stderr).unwrap());
    // io::stdout().write_all(&output.stdout).unwrap();
    // io::stderr().write_all(&output.stderr).unwrap();
    let path = Path::new("utils/gdal/gdalwarp.exe").canonicalize().unwrap();
    let output = Command::new(path)
        .args([
            "-r",
                  "lanczos",
            "-tps",
            "-co",
                  "COMPRESS=NONE",
            "-t_srs",
                  "EPSG:4326",
            "D:\\Archivos\\OpenBTMapGen\\cache\\map.tif",
            "D:\\Archivos\\OpenBTMapGen\\cache\\georef.tif"
        ])
        .output().expect("Error running warp command!");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}
