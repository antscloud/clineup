use env_logger;
use photo_organizer_rs::cli::parse_cli;
use photo_organizer_rs::exif_extractor::ExifExtractor;
// use photo_organizer_rs::path_formatter::PathFormatter;
use std::fs;
fn main() {
    parse_cli();
    // fn list_files_in_folder(folder_path: &str) -> Result<(), std::io::Error> {
    //     let entries = fs::read_dir(folder_path)?;

    //     for entry in entries.take(5) {
    //         let entry = entry?;
    //         let os_file_name = entry.path();
    //         let file_name = os_file_name.to_str().unwrap();
    //         println!("{file_name}");
    //         // Check if the entry is a file
    //         if entry.file_type()?.is_file() {
    //             let path = PathFormatter::new("%year/%month/%camera_brand/%city".to_string())
    //                 .get_formatted_path(file_name.to_string());
    //             if let Ok(v) = path {
    //                 println!("{}", v);
    //             }
    //         }
    //     }

    //     Ok(())
    // }
    // list_files_in_folder("/home/agibek/Documents/Personnel/Photos/Claire");
}
