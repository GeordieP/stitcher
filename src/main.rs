#![feature(fs_try_exists)]
#![feature(exit_status_error)]

use chrono::prelude::*;
use std::{path::PathBuf, process::Command};
use clap::Parser;

#[derive(Parser, Debug)]
struct CliArgs {
    /// Directory to look for files in.
    #[arg(short, long)]
    input_path: PathBuf,

    /// (optional) Name of the output file. file type should match the input file types.
    #[arg(short, long)]
    out: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let cli_args = CliArgs::parse();

    let ffmpeg_bin_path = find_valid_ffmpeg_binary(vec![
        PathBuf::from("/bin/ffmpeg"),
        PathBuf::from("./vendor/ffmpeg/ffmpeg"),
    ])?;

    let output_file_name = match cli_args.out {
        Some(out) => out,
        None => {
            let date = Local::now().format("%d-%h-%Y %H:%M");
            PathBuf::from(format!("STITCH_OUTPUT_{}.wav", date))
        }
    };

    let files_to_stitch = look_for_files(cli_args.input_path);
    if files_to_stitch.len() == 0 {
        return Err(String::from("found no files!"));
    }

    stitch_files(ffmpeg_bin_path, output_file_name, files_to_stitch);

    Ok(())
}

//

fn find_valid_ffmpeg_binary(
    paths_to_check: Vec<std::path::PathBuf>,
) -> Result<std::path::PathBuf, String> {
    // try to run a help command, return Ok on first 0 status code
    //
    for path in &paths_to_check {
        let output = Command::new(&path).arg("-h").output();
        if output.is_ok_and(|x| x.status.success()) {
            return Ok(path.to_path_buf());
        }
    }

    Err(format!(
        "failed to find a valid ffmpeg binary. checked paths: {:?}",
        paths_to_check
    ))
}

fn look_for_files(in_path: std::path::PathBuf) -> Vec<std::path::PathBuf> {
    match std::fs::read_dir(in_path) {
        Err(_) => vec![],
        Ok(result) => result
            .into_iter()
            .filter_map(|x| x.ok())
            .map(|x| x.path())
            .filter_map(filter_supported_extensions)
            .collect(),
    }
}

fn filter_supported_extensions(path: PathBuf) -> Option<PathBuf> {
    match path.extension()?.to_str()? {
        | "mp3"
        | "wav" => Some(path),
        _ => None,
    }
}

fn stitch_files(
    ffmpeg_bin_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
    files: Vec<std::path::PathBuf>,
) -> std::path::PathBuf {
    // set up paths
    //
    let output_file_path = output_path.as_os_str();
    let inputs_file_path = "./_stitcher_tmp_.txt";

    // write each 'file to stitch' path as lines to a temporary file, for use in ffmpeg
    //
    let inputs_file_contents = {
        let mut wip = String::new();
        for file in files {
            match file.to_str() {
                Some(file) => {
                    wip.push_str("file ");
                    wip.push_str(file);
                    wip.push_str("\n");
                }
                None => panic!("failed to convert the file paths into a single string"),
            }
        }
        wip
    };

    if let Err(e) = std::fs::write(&inputs_file_path, &inputs_file_contents) {
        panic!("failed to write lines to the temp file!: {:?}", e);
    }

    // run the command
    //
    let output = Command::new(ffmpeg_bin_path)
        .arg("-y")
        .arg("-vn")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&inputs_file_path)
        .arg("-c")
        .arg("copy")
        .arg(&output_file_path)
        .status()
        .expect("did not concatenate the files: ffmpeg command failed");

    // check the result
    //
    match output.exit_ok() {
        Err(_e) => panic!("did not concatenate the files: exit not ok: {:?}", &output),
        Ok(_) => println!("successfully concatenated the files"),
    }

    // clean the temp file up
    //
    if let Err(e) = std::fs::remove_file(inputs_file_path) {
        panic!("failed to clean up the temporary file! {:?}", e);
    }

    //

    PathBuf::from(output_file_path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_finding_files() {
        let sounds_dir_path = test_path_wav();
        let files = look_for_files(sounds_dir_path);
        let expected_len = 3;
        let actual_len = files.len();
        assert!(
            actual_len == expected_len,
            "expected `look_for_files` to find {} files in the sounds directory",
            expected_len
        );
    }

    #[test]
    fn test_finding_valid_ffmpeg_binary() {
        use std::path::PathBuf;
        match find_valid_ffmpeg_binary(vec![PathBuf::from("/bin/ffmpeg"), PathBuf::from("./vendor/ffmpeg/ffmpeg")]) {
            Err(_) => panic!("test expected to receive a valid ffmpeg binary path from `find_valid_ffmpeg_binary`"),
            Ok(_path) => (),
        }
    }

    #[test]
    pub fn expensive_test_stitching_files() {
        let ffmpeg_exe_path = match find_valid_ffmpeg_binary(
            vec![PathBuf::from("/bin/ffmpeg"), PathBuf::from("./vendor/ffmpeg/ffmpeg")]) {
            Err(_) => panic!("test expected to receive a valid ffmpeg binary path from `find_valid_ffmpeg_binary`"),
            Ok(path) => path,
        };

        let input_files = look_for_files(test_path_wav());
        let expected_output_path = std::path::PathBuf::from("./TEST_OUTPUT.wav");
        let actual_output_path = stitch_files(ffmpeg_exe_path, expected_output_path.clone(), input_files);

        assert!(
            actual_output_path == expected_output_path,
            "expected `stitch_files` to produce an output file at {}, got {}",
            expected_output_path.to_string_lossy(),
            actual_output_path.to_string_lossy()
        );

        if let Err(e) = std::fs::remove_file(actual_output_path) {
            panic!("failed to clean up the `actual output path`!: {}", e);
        }
    }

    #[test]
    pub fn test_filter_supported_extensions() {
        type T = Vec<PathBuf>;

        let paths = vec!["file1.txt", "file2.wav", "file3.mp3", "file4.rs", "file5"]
            .iter()
            .map(PathBuf::from)
            .collect::<T>();

        let expected_supported = vec!["file2.wav", "file3.mp3"]
            .iter()
            .map(PathBuf::from)
            .collect::<T>();

        let actual = paths
            .into_iter()
            .filter_map(filter_supported_extensions)
            .collect::<T>();

        if !expected_supported.eq(&actual) {
            dbg!(&expected_supported);
            dbg!(&actual);
            assert!(
                false,
                "expected the filtered extensions to match the supported values"
            );
        }
    }

    //

    fn test_path_wav() -> std::path::PathBuf {
        let sounds_dir_path = std::path::PathBuf::from("./test/stitcher/sounds/wav");
        std::fs::try_exists(&sounds_dir_path)
            .expect("this test expects to be run from the project root");
        sounds_dir_path
    }

    fn _test_path_mp3() -> std::path::PathBuf {
        let sounds_dir_path = std::path::PathBuf::from("./test/stitcher/sounds/mp3");
        std::fs::try_exists(&sounds_dir_path)
            .expect("this test expects to be run from the project root");
        sounds_dir_path
    }
}
