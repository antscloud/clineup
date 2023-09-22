use std::{fs::File, path::Path};

fn copy_directory(source: &Path, target: &Path) -> std::io::Result<()> {
    let mut stack = Vec::new();
    stack.push((source.to_path_buf(), target.to_path_buf()));

    while let Some((src, dst)) = stack.pop() {
        if src.is_dir() {
            std::fs::create_dir_all(&dst)?;

            for entry in std::fs::read_dir(&src)? {
                let entry = entry?;
                let entry_src = entry.path();
                let entry_dst = dst.join(entry_src.file_name().unwrap());

                if entry_src.is_dir() {
                    stack.push((entry_src, entry_dst));
                } else if entry_src.is_file() {
                    std::fs::copy(entry_src, entry_dst)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::path::PathBuf;
    use tempdir::TempDir;
    #[test]
    fn test_cmd_help() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let assert = cmd.arg("--help").assert();
        assert.success().stdout(predicates::str::contains("USAGE"));
    }
    #[test]
    fn test_cmd_missing_folder_or_filename() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let assert = cmd
            .arg("--source=whatever")
            .arg("--destination=whatever")
            .assert();
        assert.failure().stdout(predicates::str::contains(
            "You should provide at least one of the folder or filename format.",
        ));
    }

    #[test]
    fn test_cmd_missing_folder_or_filename_but_verbose() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let assert = cmd
            .arg("--source=whatever")
            .arg("--destination=whatever")
            .arg("-vv")
            .assert();
        assert
            .failure()
            .stderr(predicates::str::contains("DEBUG"))
            .stderr(predicates::str::contains("Get config"));
    }
    #[test]
    fn test_cmd_copy_strategy() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/");
        let input_tmp_dir = TempDir::new("input_copy_strategy").unwrap();
        let output_tmp_dir = TempDir::new("output_copy_strategy").unwrap();

        // Copy content of data folder inside tmp dir :
        copy_directory(&data_path, &input_tmp_dir.path()).unwrap();
        let assert = cmd
            .arg(format!(
                "--source={}",
                format!("{}", input_tmp_dir.path().to_string_lossy())
            ))
            .arg(format!(
                "--destination={}",
                output_tmp_dir.path().to_string_lossy()
            ))
            .arg("--folder-format={%year}")
            .arg("--strategy=copy")
            .arg("-vv")
            .assert();
        assert.success();

        let expected_2023 = Path::new(output_tmp_dir.path())
            .join("2023")
            .join("Paris-20230304.jpg");
        assert!(expected_2023.exists());

        let expected_unknown_year = Path::new(output_tmp_dir.path())
            .join("Unknown Year")
            .join("Paris.png");
        assert!(expected_unknown_year.exists());

        let still_exist = Path::new(input_tmp_dir.path()).join("Paris-20230304.jpg");
        assert!(still_exist.exists());
        let expected_number_of_files = 3;
        let entries = std::fs::read_dir(Path::new(output_tmp_dir.path()).join("2023"))
            .unwrap()
            .count();
        assert!(entries == expected_number_of_files)
    }

    #[test]
    fn test_cmd_move_strategy() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/");
        let input_tmp_dir = TempDir::new("input_move_strategy").unwrap();
        let output_tmp_dir = TempDir::new("output_move_strategy").unwrap();

        // Copy content of data folder inside tmp dir :
        copy_directory(&data_path, &input_tmp_dir.path()).unwrap();
        let assert = cmd
            .arg(format!(
                "--source={}",
                format!("{}", input_tmp_dir.path().to_string_lossy())
            ))
            .arg(format!(
                "--destination={}",
                output_tmp_dir.path().to_string_lossy()
            ))
            .arg("--folder-format={%year}")
            .arg("--strategy=move")
            .arg("-vv")
            .assert();
        assert.success();

        let expected_2023 = Path::new(output_tmp_dir.path())
            .join("2023")
            .join("Paris-20230304.jpg");
        assert!(expected_2023.exists());
        let expected_unknown_year = Path::new(output_tmp_dir.path())
            .join("Unknown Year")
            .join("Paris.png");

        assert!(expected_unknown_year.exists());

        let should_not_exist = Path::new(input_tmp_dir.path()).join("Paris-20230304.jpg");
        assert!(!should_not_exist.exists());

        let expected_number_of_files = 3;
        let entries = std::fs::read_dir(Path::new(output_tmp_dir.path()).join("2023"))
            .unwrap()
            .count();
        assert!(entries == expected_number_of_files)
    }

    #[test]
    fn test_cmd_symlink_strategy() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/");
        let input_tmp_dir = TempDir::new("input_symlink_strategy").unwrap();
        let output_tmp_dir = TempDir::new("output_symlink_strategy").unwrap();

        // Copy content of data folder inside tmp dir :
        copy_directory(&data_path, &input_tmp_dir.path()).unwrap();
        let assert = cmd
            .arg(format!(
                "--source={}",
                format!("{}", input_tmp_dir.path().to_string_lossy())
            ))
            .arg(format!(
                "--destination={}",
                output_tmp_dir.path().to_string_lossy()
            ))
            .arg("--folder-format={%year}")
            .arg("--strategy=move")
            .arg("-vv")
            .assert();
        assert.success();

        let expected = Path::new(output_tmp_dir.path())
            .join("2023")
            .join("Paris-20230304.jpg");
        assert!(expected.exists());

        let expected_number_of_files = 3;
        let entries = std::fs::read_dir(Path::new(output_tmp_dir.path()).join("2023"))
            .unwrap()
            .count();
        assert!(entries == expected_number_of_files)
    }
    #[test]
    fn test_cmd_copy_strategy_recursive() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/");
        let input_tmp_dir = TempDir::new("input_copy_strategy").unwrap();
        let output_tmp_dir = TempDir::new("output_copy_strategy").unwrap();

        // Copy content of data folder inside tmp dir :
        copy_directory(&data_path, &input_tmp_dir.path()).unwrap();
        let assert = cmd
            .arg(format!(
                "--source={}",
                format!("{}", input_tmp_dir.path().to_string_lossy())
            ))
            .arg(format!(
                "--destination={}",
                output_tmp_dir.path().to_string_lossy()
            ))
            .arg("--folder-format={%year}")
            .arg("--strategy=copy")
            .arg("--recursive")
            .arg("-vv")
            .assert();
        assert.success();
        for folder in 2013..2023 {
            let path = Path::new(output_tmp_dir.path()).join(format!("{}", folder));
            assert!(path.exists())
        }
    }
    #[test]
    fn test_cmd_copy_strategy_recursive_exclude_regex() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/");
        let input_tmp_dir = TempDir::new("input_copy_strategy").unwrap();
        let output_tmp_dir = TempDir::new("output_copy_strategy").unwrap();

        // Copy content of data folder inside tmp dir :
        copy_directory(&data_path, &input_tmp_dir.path()).unwrap();
        let assert = cmd
            .arg(format!(
                "--source={}",
                format!("{}", input_tmp_dir.path().to_string_lossy())
            ))
            .arg(format!(
                "--destination={}",
                output_tmp_dir.path().to_string_lossy()
            ))
            .arg("--folder-format={%year}")
            .arg("--strategy=copy")
            .arg("--recursive")
            .arg("--exclude-regex=.*Paris-20130101.jpg.*")
            .arg("-vv")
            .assert();
        assert.success();
        let should_not_exist = Path::new(output_tmp_dir.path()).join("2013");
        assert!(!should_not_exist.exists());
        for folder in 2014..2023 {
            let path = Path::new(output_tmp_dir.path()).join(format!("{}", folder));
            assert!(path.exists())
        }
    }
    #[test]
    fn test_cmd_copy_strategy_recursive_include_regex() {
        let mut cmd = Command::cargo_bin("clineup").unwrap();
        let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/");
        let input_tmp_dir = TempDir::new("input_copy_strategy").unwrap();
        let output_tmp_dir = TempDir::new("output_copy_strategy").unwrap();

        // Copy content of data folder inside tmp dir :
        copy_directory(&data_path, &input_tmp_dir.path()).unwrap();
        let assert = cmd
            .arg(format!(
                "--source={}",
                format!("{}", input_tmp_dir.path().to_string_lossy())
            ))
            .arg(format!(
                "--destination={}",
                output_tmp_dir.path().to_string_lossy()
            ))
            .arg("--folder-format={%year}")
            .arg("--strategy=copy")
            .arg("--recursive")
            .arg("--include-regex=.*Paris-20130101.jpg.*")
            .arg("-vv")
            .assert();
        assert.success();
        let should_exist = Path::new(output_tmp_dir.path()).join("2013");
        assert!(should_exist.exists());

        // All the others should not exist
        for folder in 2014..2023 {
            let path = Path::new(output_tmp_dir.path()).join(format!("{}", folder));
            assert!(!path.exists())
        }
    }
}
