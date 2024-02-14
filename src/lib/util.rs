//! Convenience functions for usage

use tracing_subscriber::{
    fmt::{self, format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

use tracing_log::NormalizeEvent;

use ansi_term::{Color, Style};
use color_eyre::eyre::WrapErr;
use fmt::{time::FormatTime, FormattedFields};
use std::fmt::Write;
use tracing::{
    field::{Field, Visit},
    Level, Subscriber,
};

/// Build a logger that does file and term logging.
pub fn build_logger() -> Result<(), color_eyre::eyre::Error> {
    use tracing_subscriber::prelude::__tracing_subscriber_field_MakeExt as _;

    tracing_log::log_tracer::Builder::new()
        .init()
        .context("when building tracing builder")?;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        .add_directive("rustyline=warn".parse()?);


    let subscriber = tracing_subscriber::fmt::fmt()
        .with_target(true)
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .pretty()
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("could not set global tracing logger")?;
    Ok(())
}
