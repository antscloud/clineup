#[derive(Debug, Deserialize, Serialize)]
enum OrganizationMode {
    Symlinks,
    InPlace,
    Copy,
}

trait Strategy {
    pub fn organize() {}
}

struct CopyStrategy {}
struct SymlinksStrategy {}
struct InplaceStrategy {}
