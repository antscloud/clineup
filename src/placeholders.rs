use serde::{Deserialize, Serialize};

// Configuration struct for the photo organizer
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Placeholder {
    Year,
    Month,
    Day,
    CTimeYear,
    CTimeMonth,
    CTimeDay,
    MTimeYear,
    MTimeMonth,
    MTimeDay,
    Width,
    Height,
    CameraModel,
    CameraBrand,
    Country,
    State,
    County,
    Municipality,
    City,
    OriginalFolder,
    OriginalFilename,
    Unknown,
    Fallback,
}

impl Placeholder {
    pub fn from_string<S: Into<String>>(chain: S) -> Placeholder {
        let format_string = chain.into();
        let format_string_str = format_string.as_str();
        match format_string_str {
            "%year" => Placeholder::Year,
            "%month" => Placeholder::Month,
            "%day" => Placeholder::Day,
            "%ctimeyear" => Placeholder::CTimeYear,
            "%ctimemonth" => Placeholder::CTimeMonth,
            "%ctimeday" => Placeholder::CTimeDay,
            "%mtimeyear" => Placeholder::MTimeYear,
            "%mtimemonth" => Placeholder::MTimeMonth,
            "%mtimeday" => Placeholder::MTimeDay,
            "%width" => Placeholder::Width,
            "%height" => Placeholder::Height,
            "%camera_model" => Placeholder::CameraModel,
            "%camera_brand" => Placeholder::CameraBrand,
            "%country" => Placeholder::Country,
            "%state" => Placeholder::State,
            "%county" => Placeholder::County,
            "%municipality" => Placeholder::Municipality,
            "%city" => Placeholder::City,
            "%original_folder" => Placeholder::OriginalFolder,
            "%original_filename" => Placeholder::OriginalFilename,
            _ if format_string.starts_with("%") => Placeholder::Unknown,
            _ => Placeholder::Fallback,
        }
    }

    pub fn is_exif_related(&self) -> bool {
        match self {
            Placeholder::Year
            | Placeholder::Month
            | Placeholder::Day
            | Placeholder::Width
            | Placeholder::Height
            | Placeholder::CameraModel
            | Placeholder::CameraBrand
            | Placeholder::Country
            | Placeholder::State
            | Placeholder::County
            | Placeholder::Municipality
            | Placeholder::City => true,
            _ => false,
        }
    }
    pub fn is_os_related(&self) -> bool {
        match self {
            Placeholder::CTimeYear
            | Placeholder::CTimeMonth
            | Placeholder::CTimeDay
            | Placeholder::MTimeYear
            | Placeholder::MTimeMonth
            | Placeholder::MTimeDay => true,
            _ => false,
        }
    }
    pub fn is_location_related(&self) -> bool {
        match self {
            Placeholder::Country
            | Placeholder::State
            | Placeholder::County
            | Placeholder::Municipality
            | Placeholder::City => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        assert_eq!(Placeholder::from_string("%year"), Placeholder::Year);
        assert_eq!(Placeholder::from_string("%month"), Placeholder::Month);
        assert_eq!(Placeholder::from_string("%day"), Placeholder::Day);
        assert_eq!(Placeholder::from_string("%width"), Placeholder::Width);
        assert_eq!(Placeholder::from_string("%height"), Placeholder::Height);
        assert_eq!(
            Placeholder::from_string("%camera_model"),
            Placeholder::CameraModel
        );
        assert_eq!(
            Placeholder::from_string("%camera_brand"),
            Placeholder::CameraBrand
        );
        assert_eq!(Placeholder::from_string("%country"), Placeholder::Country);
        assert_eq!(Placeholder::from_string("%state"), Placeholder::State);
        assert_eq!(Placeholder::from_string("%county"), Placeholder::County);
        assert_eq!(
            Placeholder::from_string("%municipality"),
            Placeholder::Municipality
        );
        assert_eq!(Placeholder::from_string("%city"), Placeholder::City);
        assert_eq!(
            Placeholder::from_string("%original_folder"),
            Placeholder::OriginalFolder
        );
        assert_eq!(
            Placeholder::from_string("%original_filename"),
            Placeholder::OriginalFilename
        );
        assert_eq!(
            Placeholder::from_string("%unknown_placeholder"),
            Placeholder::Unknown
        );
        assert_eq!(Placeholder::from_string("fallback"), Placeholder::Fallback);
    }
}
