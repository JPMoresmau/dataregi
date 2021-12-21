use sloggers::Build;
use sloggers::terminal::{TerminalLoggerBuilder, Destination};
use sloggers::types::Severity;
use slog_scope;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Info);
    builder.destination(Destination::Stderr);
    let logger = builder.build().unwrap();
    let _guard1=sloggers::set_stdlog_logger(logger.clone()).unwrap();
    let _guard2=slog_scope::set_global_logger(logger);

    dataregi::rocket().launch().await
}