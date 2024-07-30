use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use zip::read::ZipArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pw_file_path = Path::new("./data/Ashley_Madison.txt");
    let zip_file_path = Path::new("./data/protected.zip");

    // Ensure the password file exists
    let pw_file = match File::open(pw_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening password file: {}", e);
            return Err(Box::new(e));
        }
    };
    let reader = BufReader::new(pw_file);

    // Measure start time
    let start = Instant::now();

    // Try each password
    for line in reader.lines() {
        match line {
            Ok(password) => {
                let password = password.trim().to_string();
                if try_password(&zip_file_path, &password).is_ok() {
                    println!("Files extracted successfully with password: {}", password);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    // Measure end time and print duration
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}
fn try_password(zip_path: &Path, password: &str) -> zip::result::ZipResult<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Attempt to extract the archive
    for i in 0..archive.len() {
        let mut file = match archive.by_index_decrypt(i, password.as_bytes()) {
            Ok(Ok(f)) => f,
            _ => return Err(zip::result::ZipError::FileNotFound), // Return error if the password is incorrect
        };

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}