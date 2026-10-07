use super::{Category, LogStore, Severity};
use std::{fmt, sync::Arc};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

struct NativeLogger(Arc<LogStore>);

impl log::Log for NativeLogger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let level = match record.level() {
            log::Level::Trace => Severity::Trace,
            log::Level::Debug => Severity::Debug,
            log::Level::Info => Severity::Info,
            log::Level::Warn => Severity::Warn,
            log::Level::Error => Severity::Error,
        };
        // Logging itself must never call log!/tracing! recursively.
        if let Err(error) = self.0.record(
            level,
            category(record.target()),
            record.target(),
            record.args().to_string(),
        ) {
            eprintln!("{error}");
        }
    }

    fn flush(&self) {}
}

fn category(target: &str) -> Category {
    if target.starts_with("financiallifeplansimulator::settings") {
        Category::Setting
    } else if target.starts_with("financiallifeplansimulator") {
        Category::Common
    } else {
        Category::External
    }
}

#[derive(Default)]
struct Fields(String);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        if field.name() == "message" {
            self.0.push_str(&format!("{value:?}"));
        } else {
            self.0.push_str(&format!("{}={value:?}", field.name()));
        }
    }
}

struct NativeLayer(Arc<LogStore>);

impl<S: Subscriber> Layer<S> for NativeLayer {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            tracing::Level::TRACE => Severity::Trace,
            tracing::Level::DEBUG => Severity::Debug,
            tracing::Level::INFO => Severity::Info,
            tracing::Level::WARN => Severity::Warn,
            tracing::Level::ERROR => Severity::Error,
        };
        let mut fields = Fields::default();
        event.record(&mut fields);
        if let Err(error) = self.0.record(
            level,
            category(metadata.target()),
            metadata.target(),
            fields.0,
        ) {
            eprintln!("{error}");
        }
    }
}

pub fn install(store: Arc<LogStore>) -> Result<(), String> {
    log::set_boxed_logger(Box::new(NativeLogger(store.clone())))
        .map_err(|error| error.to_string())?;
    log::set_max_level(log::LevelFilter::Trace);
    let subscriber = tracing_subscriber::registry().with(NativeLayer(store));
    tracing::subscriber::set_global_default(subscriber).map_err(|error| error.to_string())
}
