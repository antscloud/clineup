# Clineup

Clineup (pronounced "clean up") is the fusion of "CLI" (Command-Line Interface) and "clean up."

It is a Rust-based CLI utility aimed at providing pragmatic programmable media rearrangement.

Inspired by [Elodie](https://github.com/jmathai/elodie) but with some differences and extra-features : 

- Multiple reverse geocoding API
- Multiple variables fallback 
- Ability to Copy / Move or Symlink
- Written in Rust

## Installation 

### cargo (recommended)

```sh
cargo install clineup
```

### Download binary from release

You can download the pre-built binary for your platform from the [Release page](https://github.com/antscloud/clineup/releases)

For example, for Linux, you can use the following command :

```
curl -L https://github.com/antscloud/clineup/releases/download/<clineup-version>/clineup-<clineup-version>-ubuntu-latest.tar.gz | tar -xz -C <where-you-want-to-install-it>
```

Don't forget to add <where-you-want-to-install-it> to your PATH so that you can use it from anywhere you want
### docker 

```sh
docker run -v <ABSOLUTE-PATH-OF-YOUR-SOURCE-FOLDER>:/source -v <ABSOLUTE-PATH-OF-YOUR-DESTINATION-FOLDER>:/destination antscloud/clineup --source /source --destination /destination [YOUR-OPTIONS-HERE]
```

## Usage

```sh
Clineup 
Utility tool for organizing media

USAGE:
    clineup [FLAGS] [OPTIONS] --destination <DESTINATION> --source <SOURCE>

FLAGS:
        --drop-duplicates            
            Drop duplicates depending on the strategy 
            
                            - Copy : Do not copy the duplicates 
            
                            - Symlink : Do not symlink the duplicates 
            
                            - Move : Do not move the duplicates
                            
        --dry-run                    
            Performs a dry run without actually moving or renaming any files

        --dry-run-number-of-files    
            Specifies the number of files to be processed by the dry run

        --folder-format              
            Specifies the folder format to create

        --gps-optimization           
            Round the lat ang long to 1 decimal places. It becomes less accurate (about 1 kilometer) but can save a lot
            of API calls.
    -h, --help                       
            Prints help information

        --recursive                  
            Performs the organization process recursively on subdirectories

    -V, --version                    
            Prints version information

    -v                               
            Sets the log level to increase verbosity


OPTIONS:
        --destination <DESTINATION>
            Specifies the destination directory where the organized photos will be stored

        --exclude-extension <EXTENSION>            
            Excludes photos with the specified file extensions

        --exclude-regex <EXCLUDE-REGEX>            
            
                            The regex is matched against the full path of the file, including the parent folders.
            
                            For example, to exclude all files containing 'IMG', use the regex '.*IMG.*
        --extension <EXTENSION>                    
            Filters photos based on file extensions

        --filename-format <filename-format>        
            Specifies the filename format to create

        --include-regex <INCLUDE-REGEX>            
            
                            The regex is matched against the full path of the file, including the parent folders.
            
                            For example, to include all files containing 'IMG', use the regex '.*IMG.*
        --nominatim-email <nominatim-email>
            Email to use for nominatim API. This is mandatory following the nominatim usage policy

        --reverse-geocoding <reverse-geocoding>    
            Reverse geocoding provider to use [possible values: nominatim]

        --size-greater <SIZE>
            Filters photos greater than the specified size. Use 'KB', 'MB', 'GB', 'TB' or 'PB'

        --size-lower <SIZE>
            Filters photos lower than the specified size. Use 'KB', 'MB', 'GB', 'TB' or 'PB'

        --source <SOURCE>                          
            Specifies the source directory or file to be organized

        --strategy <strategy>
            Specifies the organization strategy [default: copy]  [possible values: copy, symlink, move]
```
## Tags 

| Tag                | Meaning                                     |
|--------------------|---------------------------------------------|
| %year              | Year of the modification date               |
| %month             | Month of the modification date              |
| %day               | Day of the modification date                |
| %width             | Width of the media                          |
| %height            | Height of the media                         |
| %camera_model      | Camera model                                |
| %camera_brand      | Camera brand                                |
| %country           | Country where the photo was taken           |
| %state             | State where the photo was taken             |
| %county            | county where the photo was taken            |
| %municipality      | municipality where the photo was taken      |
| %city              | city where the photo was taken              |
| %original_folder   | Original folder where the media is          |
| %original_filename | Original filename of the media              |
| %ctimeyear         | Year of the creation date of the media      |
| %ctimemonth        | Month of the creation date of the media     |
| %ctimeday          | Day of the creation date of the media       |
| %mtimeyear         | Year of the modification date of the media  |
| %mtimemonth        | Month of the modification date of the media |
| %mtimeday          | Day of the modification date of the media   |

## Syntax

The syntax to respect is the following : 

1. **Direct Placeholder**: You can use the % symbol followed by the tag name directly, like %year or %month.

2. **Escaped Placeholder**: If you want to escape the tag and prevent unintended interpretation, you can use curly braces {}. For example, {%year}_{%month} will be treated as separate placeholders %year and %month.

3. **Fallback Placeholder**: You can define fallback values for a placeholder using the pipe | symbol. If the primary tag fails to be found, the library will automatically try the next fallback tag, and so on. If all fallbacks fail, the library will use the specified fallback string. For example, {%year|%camera_brand|Unknown year} will try %year, then %camera_brand, and finally, if both fail, it will use the fallback "Unknown year".

### Example

`{%year}/{%month|Custom month}/%camera_brand/{%city|Unknown city}` could be replaced these ways : 

- `2023/08/SomeBrand/Paris`
- `2023/08/SomeBrand/Unknown city`
- `2023/08/Unknown camera brand/Unknown city`
- `Unknown year/Custom month/Unknown camera brand/Unknown city`
- etc

## TODO 

- [ ] Implements other reverse geocoding services
- [ ] Add event placeholder
- [ ] Exclude or include pattern for files
- [ ] Add tests 
- [ ] Add TOML config file