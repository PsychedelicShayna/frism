use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::str::FromStr;

const USAGE: &str = "

    Usage: frism <split|join> filename.ext    <N[k|m|g]>    [outfile.ext]
                                              ----------    ------------
                                                (split)        (join)

            Splitting

  $ frism split filename.ext 50m         | All whitespace is eliminated after the
  $ frism split - filename.ext 50m       | filename, and treated as one argument, 
    .................................... | This would still be valid: 1 00 0 000 k 
 >> filename.ext.0, filename.ext.1, etc  | If - is provided as the filename, then
                                         | bytes are read from stdin, and the third
                                         | argument becomes the filename template.

             Joining                      

  $ frism join filename.ext  |  Notice the lack of '.0' at the end; it's no mistake.
    .......................  |  When joining parts, the parts are found automatically
 >> filename.ext             |  by adding 0..inf to the basename until file not found.
                             |  
                             |  The output is then written as the basename. Adding a
                             |  a second filename after the basename will make that 
                             |  the output file, rather than the basename itself.


";

fn frism_split_file(filename: &str, chunk_size: usize) -> io::Result<()> {
    let mut in_file = File::open(filename)?;
    let mut chunk_buffer = vec![0u8; chunk_size];
    let mut part = 0;

    let file_size: usize = in_file
        .seek(SeekFrom::End(0))
        .expect("Failed to seek to end of file; can't determine size.")
        as usize;

    let _ = in_file.seek(SeekFrom::Start(0));
    let mut total_read = 0;

    loop {
        let bytes_read = in_file.read(&mut chunk_buffer)?;

        if bytes_read == 0 {
            break;
        }

        total_read += bytes_read;

        let output_filename = format!("{}.{}", filename, part);
        let mut output_file = File::create(&output_filename)?;
        output_file.write_all(&chunk_buffer[..bytes_read])?;

        print!(
            "({:.2}%) Wrote {}{}\r",
            (total_read as f64 / file_size as f64) * 100.0,
            &output_filename,
            " ".repeat(output_filename.len())
        );

        let _ = std::io::stdout().flush();

        part += 1;
    }

    println!();

    Ok(())
}

fn frism_split_bytes(data: Vec<u8>, filename: &str, chunk_size: usize) -> io::Result<()> {
    let mut part = 0;
    let mut index = 0;

    while index < data.len() {
        let chunk_size = std::cmp::min(chunk_size, data.len() - index);
        let chunk = &data[index..index + chunk_size];

        let output_filename = format!("{}.{}", filename, part);
        let mut output_file = File::create(output_filename)?;
        output_file.write_all(chunk)?;

        index += chunk_size;
        part += 1;
    }

    Ok(())
}

fn frism_join_file(filename: &String, outfile_path: Option<String>) -> io::Result<()> {
    let first_part_path = Path::new(filename);

    let outfile_path: String = outfile_path.unwrap_or(
        first_part_path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_string)
            .unwrap(),
    );

    let mut out_file = File::create(&outfile_path)?;
    let mut part = 0;

    loop {
        let part_file_name = format!("{}.{}", filename, part);

        // If the fileanme + the part number doesn't exist, there must be
        // no more parts left, so just break.
        if !Path::new(&part_file_name).exists() {
            break;
        }

        part += 1;

        let mut part_file = File::open(&part_file_name)?;
        let mut buffer = Vec::new();

        part_file.read_to_end(&mut buffer)?;
        out_file.write_all(&buffer)?;

        print!(
            // if you need more than 7 digits to represent how many parts
            // you've split your file into.. maybe it's time to reconsider.
            //        vvvvvv
            "Joined {}       \r",
            part_file_name,
        );

        let _ = std::io::stdout().flush();
    }

    println!("Wrote to {}", &outfile_path);

    Ok(())
}

fn parse_size_suffix(size_str: &str) -> usize {
    let multiplier = match size_str
        .chars()
        .last()
        .as_ref()
        .map(char::to_ascii_lowercase)
    {
        Some('g') => 1024 * 1024 * 1024,
        Some('m') => 1024 * 1024,
        Some('k') => 1024,
        _ => 1,
    };

    let number_str = size_str.trim_end_matches(char::is_alphabetic);

    let number = usize::from_str(number_str)
        .unwrap_or_else(|e| panic!("Invalid size argument \"{}\", err: {:?}", size_str, e));

    number * multiplier
}

fn read_stdin() -> Vec<u8> {
    let mut buffer = Vec::new();

    std::io::stdin()
        .read_to_end(&mut buffer)
        .expect("Failed to read bytes from stdin!");

    buffer
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "split" => {
            let filename = &args[2];

            if filename != "-" {
                let size_str = args[3..].join("");
                let size = parse_size_suffix(&size_str);
                frism_split_file(filename, size).expect("Failed to split from file.");
            } else if args.len() < 4 {
                eprintln!("{}", USAGE);
                std::process::exit(1);
            } else {
                let bytes = read_stdin();
                let size_str = args[4..].join("");
                let size = parse_size_suffix(&size_str);
                frism_split_bytes(bytes, &args[3], size).expect("Failed to split from stdin.");
            }
        }
        "join" => {
            let basename = &args[2];
            let outfile = args.get(3).cloned();
            frism_join_file(basename, outfile).expect("Failed to join files");
        }
        _ => {
            eprintln!("{}\nInvalid command. Use 'split' or 'join'", USAGE);
            std::process::exit(1);
        }
    }
}
