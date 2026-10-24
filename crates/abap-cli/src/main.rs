use std::process::ExitCode;

mod mcp;

use abap_analysis::abap_source::{AdtOptions, ProviderOptions, SourceRef};
use abap_analysis::text;

const USAGE: &str = "usage: abap <command> [arguments]

single file:
  tokens <file> [--json]      tokens
    statements <file> [--json]  statements with their kind, the release each
                              needs and syntax errors found
  release <file> [--release R] [--json]
                              the ABAP release the file needs, the statements
                              that need more than R (e.g. 750, 740sp05) and
                              those with a syntax error
  symbols <file>              classes, interfaces, routines and declarations
  db <file>                   database accesses: tables, fields, joins, conditions
  calls <file>                calls: methods, function modules, forms, programs
  logic <file>                per procedure: statements with conditions, data
                              read and written, control flow, effects
  cds <file>                  CDS data definition: sources, joins, associations,
                              parameters, the fields of each element

across objects, sources from folders and/or an SAP system:
  graph <object>... [--depth N]   call graph from the given objects
  logic <object> [--depth N]      logic of all sources of an object, with the
                                  global data of its includes; calls into
                                  other objects followed N hops (default 1)
  fetch <object>                  print the source of an object
  objects are written KIND:NAME, e.g. CLAS:ZCL_ORDERS, PROG:ZREPORT, FUNC:Z_FM,
  INTF:ZIF_X, FUGR:ZGROUP, CLAS:ZCL_ORDERS/implementations; dictionary types
  TABL:BAPIRET2 (structures, tables) and DTEL:XUBNAME, and CDS data
  definitions DDLS:I_COUNTRY can be fetched

source options:
  --src <dir>          folder with abapGit or .abap files, repeatable
  --adt-url <url>      SAP system, e.g. https://host:44300 (or ABAP_ADT_URL)
  --client <nnn>       SAP client (or ABAP_ADT_CLIENT)
  --language <xx>      logon language
  --cache <dir>        cache folder for sources from the SAP system
  --insecure           skip TLS certificate checks (test systems only)
  credentials are read from ABAP_ADT_USER and ABAP_ADT_PASSWORD

following calls into other objects (graph, logic, mcp):
  --follow <pattern>   load only objects whose name matches, like Z* or a
                       custom namespace /ABC/*; repeatable; the objects
                       given are always loaded
  --skip <pattern>     do not load objects whose name matches, repeatable
  --limit <n>          load at most n objects
  --progress           write each object loaded to stderr
  ABAP_ADT_LOG=1 writes every ADT request to stderr: method, path, status,
  duration, the SQL of data preview queries; never credentials

for language models:
  mcp                  MCP server over stdio: the analyses as tools, sources
                       from the source options

testing:
  mock-adt --src <dir>... [--port N] [--client nnn] [--release 750]
                       serve the folders like an SAP system through ADT, for
                       tests of loading sources; port 50000 by default, logon
                       from ABAP_ADT_USER/ABAP_ADT_PASSWORD or DEVELOPER/mock;
                       --release 750 acts like SAP_BASIS 750 with its data
                       preview, dictionary and limits

output:
  JSON by default; --text for symbols, db, calls, logic, cds and graph gives a
  compact text: definitions written like ABAP, one line per call or access,
  effects with their statements and conditions, rows to find them

other:
  schema               JSON Schemas of all outputs";

struct Args {
    positional: Vec<String>,
    json: bool,
    text: bool,
    depth: u32,
    port: u16,
    client: Option<String>,
    /// what `mock-adt` acts like, the release `release` compares with
    release: Option<String>,
    options: ProviderOptions,
    follow: abap_analysis::Follow,
    progress: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        positional: Vec::new(),
        json: false,
        text: false,
        depth: 1,
        port: 50000,
        client: None,
        release: None,
        options: ProviderOptions::default(),
        follow: abap_analysis::Follow::default(),
        progress: false,
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
            "--text" => a.text = true,
            "--insecure" => insecure = true,
            "--src" => a.options.src.push(value("--src")?),
            "--cache" => a.options.cache = Some(value("--cache")?),
            "--adt-url" => adt_url = Some(value("--adt-url")?),
            "--client" => client = Some(value("--client")?),
            "--language" => language = Some(value("--language")?),
            "--release" => a.release = Some(value("--release")?),
            "--follow" => a.follow.include.push(value("--follow")?),
            "--skip" => a.follow.exclude.push(value("--skip")?),
            "--progress" => a.progress = true,
            "--limit" => {
                a.follow.limit = Some(
                    value("--limit")?
                        .parse()
                        .map_err(|_| "--limit needs a number".to_string())?,
                )
            }
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
                "release": abap_model::release_schema(),
                "db_access": abap_model::db_access_schema(),
                "calls": abap_model::calls_schema(),
                "call_graph": abap_model::call_graph_schema(),
                "logic": abap_model::logic_schema(),
                "cds": abap_model::cds_schema(),
            }));
            ExitCode::SUCCESS
        }
        "graph" | "fetch" => object_command(command, &args),
        "logic"
            if args
                .positional
                .get(1)
                .is_some_and(|a| a.parse::<SourceRef>().is_ok()) =>
        {
            object_command(command, &args)
        }
        "mock-adt" => mock_adt(&args),
        "mcp" => {
            let server = mcp::Server::new(args.options.build().map_err(|e| e.to_string()))
                .with_follow(args.follow.clone());
            let stdin = std::io::stdin();
            match server.run(stdin.lock(), std::io::stdout()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        "tokens" | "statements" | "release" | "symbols" | "db" | "calls" | "logic" | "cds" => {
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
                        "{}:{}\t{}\t{}{}",
                        s.span.start.row,
                        s.span.start.col,
                        s.kind,
                        s.text,
                        statement_notes(s)
                    );
                }
            }
        }
        "release" => {
            let target = match args.release.as_deref().map(str::parse).transpose() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(2);
                }
            };
            let report = abap_analysis::release(&source, target);
            if args.json {
                print_json(&report);
            } else {
                let needs = report.release.as_ref().map_or("7.00".to_string(), |r| {
                    format!("{} ({})", r.release, r.construct)
                });
                println!("needs {needs}");
                if !report.statements.is_empty() {
                    let target = report.target.as_deref().unwrap_or("7.00");
                    println!("\nnewer than {target}:");
                    for s in &report.statements {
                        println!("{}\t{}{}", s.span.start.row, s.text, statement_notes(s));
                    }
                }
                if !report.errors.is_empty() {
                    println!("\nsyntax errors:");
                    for s in &report.errors {
                        println!("{}\t{}{}", s.span.start.row, s.text, statement_notes(s));
                    }
                }
            }
        }
        "symbols" if args.text => print!("{}", text::symbols(&abap_analysis::symbols(&source))),
        "db" if args.text => print!("{}", text::db_access(&abap_analysis::db_access(&source))),
        "calls" if args.text => print!("{}", text::calls(&abap_analysis::calls(&source))),
        "logic" if args.text => print!("{}", text::logic(&abap_analysis::logic(&source))),
        "cds" if args.text => print!("{}", text::cds(&abap_analysis::cds(&source))),
        "cds" => print_json(&abap_analysis::cds(&source)),
        "symbols" => print_json(&abap_analysis::symbols(&source)),
        "db" => print_json(&abap_analysis::db_access(&source)),
        "calls" => print_json(&abap_analysis::calls(&source)),
        "logic" => print_json(&abap_analysis::logic(&source)),
        _ => unreachable!(),
    }
    ExitCode::SUCCESS
}

/// `  [7.40 SP02: inline declaration]`, `  [error: ...]` after a statement.
fn statement_notes(s: &abap_model::StatementInfo) -> String {
    let mut out = String::new();
    if let Some(r) = &s.release {
        out.push_str(&format!("  [{}: {}]", r.release, r.construct));
    }
    if let Some(e) = &s.error {
        out.push_str(&format!("  [error: {e}]"));
    }
    out
}

fn mock_adt(args: &Args) -> ExitCode {
    use abap_analysis::abap_source::{FsProvider, MockAdtConfig, MockAdtServer, MockRelease};
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
    if let Some(release) = &args.release {
        config.release = match release.parse::<MockRelease>() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(2);
            }
        };
    }
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
    let mut workspace = abap_analysis::Workspace::new(provider).with_follow(args.follow.clone());
    if args.progress {
        workspace =
            workspace.with_progress(Box::new(|p| eprintln!("{:>5} {}", p.loaded, p.object)));
    }
    let errors = if command == "logic" {
        let logic = workspace.logic(&refs[0], args.depth);
        if args.text {
            print!("{}", text::logic(&logic));
        } else {
            print_json(&logic);
        }
        logic.errors
    } else {
        let graph = workspace.call_graph(&refs, args.depth);
        if args.text {
            print!("{}", text::call_graph(&graph));
        } else {
            print_json(&graph);
        }
        graph.errors
    };
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        for e in &errors {
            eprintln!("{e}");
        }
        ExitCode::FAILURE
    }
}
