use std::{
    fmt::Write as _,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use reforge_bench_core::{
    BenchSnapshot, CLIENT_ERRORS_SCHEMA, SNAPSHOT_SCHEMA, diagnostics::Diagnostics,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

const MAX_DOCUMENT_BYTES: u64 = 1_048_576;
const DEMO: &str = include_str!("../examples/hardware-rea/summary.json");
const COMMAND_SCHEMA: &str = include_str!("../schemas/commands.json");
const HELP: &str = "Reforge Bench

  demo [--json]                            Read the recorded example
  validate-snapshot FILE [--json]           Check a saved bench snapshot
  diagnostics FILE                         Summarize errors without private text
  schema [commands|snapshot|diagnostics]    Print JSON Schema
  --request FILE                           Run a schema-described JSON command
  --version                                Print the version

Local file inspection only. No hardware, analysis engine or running app is controlled.";

#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "kebab-case", deny_unknown_fields)]
enum Command {
    Demo {
        #[serde(default)]
        json: bool,
    },
    ValidateSnapshot {
        path: PathBuf,
        #[serde(default)]
        json: bool,
    },
    Diagnostics {
        path: PathBuf,
    },
    Schema {
        #[serde(default)]
        kind: SchemaKind,
    },
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SchemaKind {
    #[default]
    Commands,
    Snapshot,
    Diagnostics,
}

fn read_document(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("Cannot inspect selected file: {}", error.kind()))?;
    if !metadata.is_file() {
        return Err("Select a regular JSON file".into());
    }
    let file =
        File::open(path).map_err(|error| format!("Cannot open selected file: {}", error.kind()))?;
    if !file
        .metadata()
        .map_err(|error| error.kind().to_string())?
        .is_file()
    {
        return Err("Select a regular JSON file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_DOCUMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read selected file: {}", error.kind()))?;
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err("JSON document exceeds 1 MiB".into());
    }
    Ok(bytes)
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| {
        format!(
            "Invalid JSON document ({:?}) at line {}, column {}",
            error.classify(),
            error.line(),
            error.column()
        )
    })
}

fn command(arguments: &[String]) -> Result<Command, String> {
    let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["demo"] => Ok(Command::Demo { json: false }),
        ["demo", "--json"] => Ok(Command::Demo { json: true }),
        ["validate-snapshot", path] => Ok(Command::ValidateSnapshot {
            path: path.into(),
            json: false,
        }),
        ["validate-snapshot", path, "--json"] => Ok(Command::ValidateSnapshot {
            path: path.into(),
            json: true,
        }),
        ["diagnostics", path] => Ok(Command::Diagnostics { path: path.into() }),
        ["schema"] | ["schema", "commands"] => Ok(Command::Schema {
            kind: SchemaKind::Commands,
        }),
        ["schema", "snapshot"] => Ok(Command::Schema {
            kind: SchemaKind::Snapshot,
        }),
        ["schema", "diagnostics"] => Ok(Command::Schema {
            kind: SchemaKind::Diagnostics,
        }),
        ["--request", path] => decode(&read_document(Path::new(path))?),
        _ => Err("Unknown command or arguments; use --help".into()),
    }
}

fn run(command: Command) -> Result<String, String> {
    match command {
        Command::Demo { json: as_json } => {
            let case: Value = decode(DEMO.as_bytes())?;
            if as_json {
                Ok(format!("{case:#}\n"))
            } else {
                let title = case["title"].as_str().ok_or("Invalid example title")?;
                let model = case["device_model"]
                    .as_str()
                    .ok_or("Invalid example model")?;
                let environment = &case["installed_environment"];
                let architecture = environment["architecture"]
                    .as_str()
                    .ok_or("Invalid example architecture")?;
                let os = environment["operating_system"]
                    .as_str()
                    .ok_or("Invalid example operating system")?;
                let display = &case["display_observation"];
                let drawing = display["temporary_custom_drawing"]
                    .as_str()
                    .ok_or("Invalid display observation")?;
                let restored = display["normal_display_return"]
                    .as_str()
                    .ok_or("Invalid display-return observation")?;
                let reference = &case["reference_measurement_summary"];
                let captures = reference["captures"]
                    .as_u64()
                    .ok_or("Invalid example capture count")?;
                let mean = reference["frequency_hz"]["mean"]
                    .as_f64()
                    .ok_or("Invalid example mean")?;
                let deviation = reference["frequency_hz"]["sample_standard_deviation"]
                    .as_f64()
                    .ok_or("Invalid example deviation")?;
                Ok([
                    title.to_owned(),
                    format!("{model} · recorded summary, not live"),
                    format!("{architecture} · {os} observed on one unit"),
                    format!("Custom drawing: {drawing} · normal display return: {restored}"),
                    format!("{captures} saved captures · mean {mean:.6} Hz · sample SD {deviation:.6} Hz"),
                    "Raw artifacts omitted. Voltage conversion and absolute calibration are unqualified.".into(),
                ].join("\n") + "\n")
            }
        }
        Command::ValidateSnapshot {
            path,
            json: as_json,
        } => {
            let snapshot: BenchSnapshot = decode(&read_document(&path)?)?;
            snapshot.validate()?;
            if as_json {
                let result = json!({"valid":true,"schema_version":snapshot.schema_version,
                    "declared_source":snapshot.source,"instruments":snapshot.instruments.len(),
                    "warnings":snapshot.warnings.len()});
                Ok(format!("{result}\n"))
            } else {
                Ok(format!(
                    "Valid snapshot · {} instruments · {} warnings\nProvenance, freshness and calibration are not authenticated.\n",
                    snapshot.instruments.len(),
                    snapshot.warnings.len()
                ))
            }
        }
        Command::Diagnostics { path } => {
            let entries: Vec<Value> = decode(&read_document(&path)?)?;
            if entries.len() > 1024 {
                return Err("Diagnostic document exceeds 1,024 entries".into());
            }
            let mut diagnostics = Diagnostics::default();
            let mut message = String::new();
            for (index, entry) in entries.iter().enumerate() {
                message.clear();
                write!(&mut message, "{entry}").map_err(|_| "Cannot format client error")?;
                diagnostics
                    .record_client(&message)
                    .ok_or_else(|| format!("Unsupported client error at entry {}", index + 1))?;
            }
            let report = serde_json::to_string_pretty(&diagnostics.shareable_report())
                .map_err(|_| "Cannot serialize diagnostic summary")?;
            Ok(report + "\n")
        }
        Command::Schema { kind } => Ok(match kind {
            SchemaKind::Commands => COMMAND_SCHEMA,
            SchemaKind::Snapshot => SNAPSHOT_SCHEMA,
            SchemaKind::Diagnostics => CLIENT_ERRORS_SCHEMA,
        }
        .to_owned()),
    }
}

fn app() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let output = if arguments.is_empty() || arguments == ["--help"] {
        format!("{HELP}\n")
    } else if arguments == ["--version"] {
        format!("Reforge Bench {}\n", env!("CARGO_PKG_VERSION"))
    } else {
        run(command(&arguments)?)?
    };
    match io::stdout().lock().write_all(output.as_bytes()) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|error| format!("Cannot write output: {}", error.kind())),
    }
}

fn main() {
    if let Err(error) = app() {
        eprintln!("Reforge Bench: {error}");
        std::process::exit(1);
    }
}
