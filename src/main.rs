//! Syncs Users and Groups from Google Workspace to AWS SSO

#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![warn(
    clippy::cargo,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::perf,
    clippy::style
)]
#![allow(
    clippy::future_not_send,
    clippy::multiple_crate_versions,
    clippy::redundant_pub_crate,
    clippy::wildcard_dependencies
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod aws;
mod event;
mod google;
mod sync;

#[cfg(feature = "test")]
const TEST_DATA: &str = include_str!("../test.json");

const ENV_VAR_LOG_LEVEL: &str = "LOG_LEVEL";
pub(crate) const ENV_VAR_SH_GOOGLE_CREDS: &str = "SH_GOOGLE_CREDS";
pub(crate) const ENV_VAR_SH_SCIM_CREDS: &str = "SH_SCIM_CREDS";
pub(crate) const ENV_VAR_GOOGLE_API_QUERY_FOR_USERS: &str = "GOOGLE_API_QUERY_FOR_USERS";
pub(crate) const ENV_VAR_GOOGLE_API_QUERY_FOR_GROUPS: &str = "GOOGLE_API_QUERY_FOR_GROUPS";
pub(crate) const ENV_VAR_IGNORE_USERS_REGEXES: &str = "IGNORE_USERS_REGEXES";
pub(crate) const ENV_VAR_INCLUDE_USERS_REGEXES: &str = "INCLUDE_USERS_REGEXES";
pub(crate) const ENV_VAR_IGNORE_GROUPS_REGEXES: &str = "IGNORE_GROUPS_REGEXES";
pub(crate) const ENV_VAR_INCLUDE_GROUPS_REGEXES: &str = "INCLUDE_GROUPS_REGEXES";
pub(crate) const ENV_VAR_SYNC_STRATEGIE: &str = "SYNC_STRATEGIE";

struct Runner;

#[async_trait::async_trait]
impl lambda_runtime_types::Runner<(), event::Event, ()> for Runner {
    async fn run<'a>(_shared: &'a (), event: event::Event, _region: &'a str) -> anyhow::Result<()> {
        let security_hub_google_creds: google::AdminCreds =
            aws::get_secret_from_secret_manager(event.get_security_hub_google_creds()?.as_ref())
                .await?;
        let security_hub_scim_creds: aws::ScimCreds =
            aws::get_secret_from_secret_manager(event.get_security_hub_scim_creds()?.as_ref())
                .await?;

        let scim = aws::Scim::new(&security_hub_scim_creds);
        let gadmin = google::Admin::new(&security_hub_google_creds).await?;

        let mut sync_op = sync::SyncOp::new(&event, &scim, &gadmin).await?;
        sync_op.sync_groups().await?;
        sync_op.sync_users(event.get_sync_strategie()?).await?;
        sync_op.sync_associations().await?;
        Ok(())
    }

    async fn setup() -> anyhow::Result<()> {
        use anyhow::Context;
        use std::str::FromStr;

        let log_level = std::env::var(ENV_VAR_LOG_LEVEL);
        let log_level = log_level.as_ref().map(AsRef::as_ref).unwrap_or("info");
        let log_level = log::LevelFilter::from_str(log_level)
            .with_context(|| format!("Invalid log_level: {}", log_level))?;
        simple_logger::SimpleLogger::new()
            .with_level(log_level)
            .init()
            .expect("Unable to setup logging");
        Ok(())
    }
}

/// Entrypoint for the lambda
pub fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "test"))]
    {
        lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
    }
    #[cfg(feature = "test")]
    {
        lambda_runtime_types::exec_test::<_, _, Runner, _>(TEST_DATA)
    }
}
