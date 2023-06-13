use async_graphql::SimpleObject;

#[derive(Clone, Debug, SimpleObject)]
pub struct VersionInfo {
    pub major: u8,
    pub minor: u8,
    pub bug_fix: u8,
    pub version_string: &'static str,
}
