use std::process::ExitCode;

use abap_analysis::abap_source::{AdtOptions, ProviderOptions, SourceRef};

const USAGE: &str = "usage: abap <command> [arguments]

single file:
  tokens <file> [--json]      tokens
  statements <file> [--json]  statements with their kind
  symbols <file>              classes, interfaces, routines and declarations
  db <file>                   database accesses: tables, fields, joins, conditions
  calls <file>                calls: methods, function modules, forms, programs
  logic <file>                per procedure: statements with conditions, data
                              read and written, control flow, effects

across objects, sources from folders and/or an SAP system:
  graph <object>... [--depth N]   call graph from the given objects
  fetch <object>                  print the source of an object
  objects are written KIND:NAME, e.g. CLAS:ZCL_ORDERS, PROG:ZREPORT, FUNC:Z_FM,
  INTF:ZIF_X, FUGR:ZGROUP, CLAS:ZCL_ORDERS/implementations

source options:
  --src <dir>          folder with abapGit or .abap files, repeatable
  --adt-url <url>      SAP system, e.g. https://host:44300 (or ABAP_ADT_URL)
  --client <nnn>       SAP client (or ABAP_ADT_CLIENT)
  --language <xx>      logon language
  --cache <dir>        cache folder for sources from the SAP system
  --insecure           skip TLS certificate checks (test systems only)
  credentials are read from ABAP_ADT_USER and ABAP_ADT_PASSWORD

testing:
  mock-adt --src <dir>... [--port N] [--client nnn]
                       serve the folders like an SAP system through ADT, for
                       tests of loading sources; port 50000 by default, logon
                       from ABAP_ADT_USER/ABAP_ADT_PASSWORD or DEVELOPER/mock

other:
  schema               JSON Schemas of all outputs";

struct Args {
    positional: Vec<String>,
    json: bool,
    depth: u32,
    port: u16,
    client: Option<String>,
    options: ProviderOptions,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        positional: Vec::new(),
        json: false,
        depth: 1,
        port: 50000,
        client: None,
        options: ProviderOptions::default(),
    };
    let mut adt_url = std::env::var("ABAP_ADT_URL").ok();
    let mut client = std::env::var("ABAP_ADT_CLIENT").ok();
    let mut language = None;
    let mut insecure = false;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "--json" => a.json = true,
            "--insecure" => insecure = true,
            "--src" => a.options.src.push(value("--src")?),
            "--cache" => a.options.cache = Some(value("--cache")?),
            "--adt-url" => adt_url = Some(value("--adt-url")?),
            "--client" => client = Some(value("--client")?),
            "--language" => language = Some(value("--language")?),
            "--port" => {
                a.port = value("--port")?
                    .parse()
                    .map_err(|_| "--port needs a number".to_string())?
            }
            "--depth" => {
                a.depth = value("--depth")?
                    .parse()
                    .map_err(|_| "--depth needs a number".to_string())?
            }
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            _ => a.positional.push(arg),
        }
    }
    a.client = client.clone();
    if let Some(url) = adt_url {
        a.options.adt = Some(AdtOptions {
            url,
            user: std::env::var("ABAP_ADT_USER").unwrap_or_default(),
            password: std::env::var("ABAP_ADT_PASSWORD").unwrap_or_default(),
            client,
            language,
            insecure,
        });
    }
    Ok(a)
}

fn print_json(value: &impl serde::Serialize) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let Some(command) = args.positional.first().map(String::as_str) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    match command {
        "schema" => {
            print_json(&serde_json::json!({
                "symbols": abap_model::symbols_schema(),
                "statements": abap_model::statements_schema(),
                "db_access": abap_model::db_access_schema(),
                "calls": abap_model::calls_schema(),
                "call_graph": abap_model::call_graph_schema(),
                "logic": abap_model::logic_schema(),
            }));
            ExitCode::SUCCESS
        }
        "graph" | "fetch" => object_command(command, &args),
        "mock-adt" => mock_adt(&args),
        "tokens" | "statements" | "symbols" | "db" | "calls" | "logic" => {
            file_command(command, &args)
        }
        _ => {
            eprintln!("unknown command {command}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn file_command(command: &str, args: &Args) -> ExitCode {
    let Some(path) = args.positional.get(1) else {
        eprintln!("{command} needs a file\n\n{USAGE}");
        return ExitCode::from(2);
    };
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    match command {
        "tokens" => {
            let lexed = abap_lexer::lex(&source);
            if args.json {
                print_json(&lexed.tokens);
            } else {
                for t in &lexed.tokens {
                    println!(
                        "{}:{}\t{}\t{}",
                        t.start.row,
                        t.start.col,
                        t.abaplint_name(),
                        t.text
                    );
                }
            }
        }
        "statements" => {
            let statements = abap_analysis::statements(&source);
            if args.json {
                print_json(&statements);
            } else {
                for s in &statements {
                    println!(
                        "{}:{}\t{}\t{}",
                        s.span.start.row, s.span.start.col, s.kind, s.text
                    );
                }
            }
        }
        "symbols" => print_json(&abap_analysis::symbols(&source)),
        "db" => print_json(&abap_analysis::db_access(&source)),
        "calls" => print_json(&abap_analysis::calls(&source)),
        "logic" => print_json(&abap_analysis::logic(&source)),
        _ => unreachable!(),
    }
    ExitCode::SUCCESS
}

fn mock_adt(args: &Args) -> ExitCode {
    use abap_analysis::abap_source::{FsProvider, MockAdtConfig, MockAdtServer};
    if args.options.src.is_empty() {
        eprintln!("mock-adt needs --src folders\n\n{USAGE}");
        return ExitCode::from(2);
    }
    let roots = args.options.src.iter().map(Into::into).collect();
    let provider = match FsProvider::new(roots) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let mut config = MockAdtConfig::default();
    if let Ok(user) = std::env::var("ABAP_ADT_USER") {
        config.user = user;
    }
    if let Ok(password) = std::env::var("ABAP_ADT_PASSWORD") {
        config.password = password;
    }
    config.client = args.client.clone();
    let user = config.user.clone();
    match MockAdtServer::start(provider, &format!("127.0.0.1:{}", args.port), config) {
        Ok(server) => {
            eprintln!(
                "ADT mock at {} serving {} objects from {}, user {user}; stop with Ctrl+C",
                server.url(),
                server.objects(),
                args.options.src.join(", ")
            );
            server.wait();
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn object_command(command: &str, args: &Args) -> ExitCode {
    let refs: Result<Vec<SourceRef>, String> =
        args.positional[1..].iter().map(|s| s.parse()).collect();
    let refs = match refs {
        Ok(r) if !r.is_empty() => r,
        Ok(_) => {
            eprintln!("{command} needs objects like CLAS:ZCL_X\n\n{USAGE}");
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let provider = match args.options.build() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if command == "fetch" {
        let mut status = ExitCode::SUCCESS;
        for r in &refs {
            match provider.fetch(r) {
                Ok(Some(s)) => print!("{}", s.text),
                Ok(None) => {
                    eprintln!("{r}: not found in {}", provider.describe());
                    status = ExitCode::FAILURE;
                }
                Err(e) => {
                    eprintln!("{r}: {e}");
                    status = ExitCode::FAILURE;
                }
            }
        }
        return status;
    }
    let workspace = abap_analysis::Workspace::new(provider);
    let graph = workspace.call_graph(&refs, args.depth);
    print_json(&graph);
    if graph.errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        for e in &graph.errors {
            eprintln!("{e}");
        }
        ExitCode::FAILURE
    }
}
