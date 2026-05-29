use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Clone, Copy)]
enum Shape {
    Wide,
    Deep,
    Keys,
}

#[derive(Clone, Copy)]
enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    fn label(self) -> &'static str {
        match self {
            Size::Small => "small",
            Size::Medium => "medium",
            Size::Large => "large",
        }
    }

    fn target_lines(self) -> usize {
        match self {
            Size::Small => 10_000,
            Size::Medium => 100_000,
            Size::Large => 1_000_000,
        }
    }

    fn keys_field_count(self) -> usize {
        match self {
            Size::Small => 1_000,
            Size::Medium => 10_000,
            Size::Large => 100_000,
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let shape = match args.get(1).map(String::as_str) {
        Some("wide") => Shape::Wide,
        Some("deep") => Shape::Deep,
        Some("keys") => Shape::Keys,
        _ => {
            print_usage();
            return ExitCode::from(2);
        }
    };

    let mut size = Size::Medium;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--size" => {
                let Some(s) = args.get(i + 1) else {
                    print_usage();
                    return ExitCode::from(2);
                };
                size = match s.as_str() {
                    "small" => Size::Small,
                    "medium" => Size::Medium,
                    "large" => Size::Large,
                    _ => {
                        eprintln!("error: unknown size '{s}' (expected small|medium|large)");
                        return ExitCode::from(2);
                    }
                };
                i += 2;
            }
            other => {
                eprintln!("error: unknown argument '{other}'");
                print_usage();
                return ExitCode::from(2);
            }
        }
    }

    let shape_label = match shape {
        Shape::Wide => "wide",
        Shape::Deep => "deep",
        Shape::Keys => "keys",
    };

    let out_path: PathBuf = [
        "tests",
        "perf_fixtures",
        &format!("{}_{}.json", shape_label, size.label()),
    ]
    .iter()
    .collect();

    let file = match File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot create {}: {e}", out_path.display());
            return ExitCode::from(1);
        }
    };
    let mut w = BufWriter::new(file);

    let result = match shape {
        Shape::Wide => write_wide(&mut w, size.target_lines()),
        Shape::Deep => write_deep(&mut w, size.target_lines()),
        Shape::Keys => write_keys(&mut w, size.keys_field_count()),
    };

    if let Err(e) = result.and_then(|_| w.flush()) {
        eprintln!("error writing fixture: {e}");
        return ExitCode::from(1);
    }

    eprintln!("wrote {}", out_path.display());
    ExitCode::SUCCESS
}

fn print_usage() {
    eprintln!("usage: gen_perf_fixture <wide|deep|keys> [--size small|medium|large]");
    eprintln!("       default size is medium (100k lines)");
    eprintln!("       small=10k lines, medium=100k lines, large=1m lines");
    eprintln!("       output goes to tests/perf_fixtures/<shape>_<size>.json");
}

const FIRST_NAMES: &[&str] = &[
    "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy", "Kevin",
    "Laura", "Mallory", "Niaj", "Olivia", "Peggy", "Quentin", "Rupert", "Sybil", "Trent",
];

const LAST_NAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Hernandez",
    "Lopez",
    "Gonzalez",
    "Wilson",
    "Anderson",
    "Thomas",
    "Taylor",
    "Moore",
    "Jackson",
    "Martin",
];

const CITIES: &[&str] = &[
    "Springfield",
    "Riverside",
    "Greenville",
    "Bristol",
    "Fairview",
    "Salem",
    "Madison",
    "Georgetown",
    "Arlington",
    "Ashland",
];

const COUNTRIES: &[&str] = &["US", "UK", "CA", "AU", "DE", "FR", "JP", "BR", "IN", "MX"];

const TAG_POOL: &[&str] = &[
    "admin", "verified", "premium", "beta", "trial", "expired", "vip", "new", "active", "pending",
    "archived", "locked",
];

fn write_wide<W: Write>(w: &mut W, target_lines: usize) -> std::io::Result<()> {
    // wide.json shape: array of objects.
    // Each object pretty-prints to ~14 lines, so object count ≈ target_lines / 14.
    // The +2 lines are the surrounding "[" and "]".
    let lines_per_object = 14;
    let count = (target_lines.saturating_sub(2)) / lines_per_object;

    writeln!(w, "[")?;
    for i in 0..count {
        let first = FIRST_NAMES[i % FIRST_NAMES.len()];
        let last = LAST_NAMES[(i / FIRST_NAMES.len()) % LAST_NAMES.len()];
        let city = CITIES[i % CITIES.len()];
        let country = COUNTRIES[i % COUNTRIES.len()];
        let tag_a = TAG_POOL[i % TAG_POOL.len()];
        let tag_b = TAG_POOL[(i + 3) % TAG_POOL.len()];
        let age = 18 + (i % 60);
        let score = ((i * 37) % 10000) as f64 / 100.0;
        let created_at = 1_700_000_000_u64 + (i as u64) * 60;
        let active = i % 3 != 0;
        // Inject occasional nulls so jiq's null path gets exercised.
        let email_block = if i % 17 == 0 {
            "null".to_string()
        } else {
            format!(
                "\"{}.{}@example.com\"",
                first.to_lowercase(),
                last.to_lowercase()
            )
        };

        writeln!(w, "  {{")?;
        writeln!(w, "    \"id\": {},", i)?;
        writeln!(w, "    \"name\": \"{} {}\",", first, last)?;
        writeln!(w, "    \"email\": {},", email_block)?;
        writeln!(w, "    \"age\": {},", age)?;
        writeln!(w, "    \"score\": {:.2},", score)?;
        writeln!(w, "    \"created_at\": {},", created_at)?;
        writeln!(w, "    \"active\": {},", active)?;
        writeln!(w, "    \"tags\": [\"{}\", \"{}\"],", tag_a, tag_b)?;
        writeln!(w, "    \"address\": {{")?;
        writeln!(w, "      \"street\": \"{} Main St\",", 100 + (i * 7) % 9900)?;
        writeln!(w, "      \"city\": \"{}\",", city)?;
        writeln!(w, "      \"country\": \"{}\"", country)?;
        if i + 1 == count {
            writeln!(w, "    }}")?;
            writeln!(w, "  }}")?;
        } else {
            writeln!(w, "    }}")?;
            writeln!(w, "  }},")?;
        }
    }
    writeln!(w, "]")?;
    Ok(())
}

fn write_deep<W: Write>(w: &mut W, target_lines: usize) -> std::io::Result<()> {
    // deep.json shape: recursively nested tree.
    // Branching factor 2-3, depth grows until total node count ≈ target_lines/5
    // (each node contributes ~5 lines: open brace, level, value, children, close brace).
    let target_nodes = target_lines / 5;
    let depth = depth_for_node_count(target_nodes);
    write_deep_node(w, 0, depth, 0, "")?;
    writeln!(w)?;
    Ok(())
}

fn depth_for_node_count(target_nodes: usize) -> usize {
    // Branching factor 3 → total nodes ≈ (3^(depth+1) - 1) / 2.
    // Solve for depth such that count >= target_nodes.
    let mut depth = 1;
    let mut total: u64 = 1;
    let mut at_level: u64 = 1;
    while total < target_nodes as u64 && depth < 14 {
        at_level *= 3;
        total += at_level;
        depth += 1;
    }
    depth
}

fn write_deep_node<W: Write>(
    w: &mut W,
    level: usize,
    max_depth: usize,
    sibling_idx: usize,
    indent: &str,
) -> std::io::Result<()> {
    let value = format!("node-{}-{}", level, sibling_idx);
    let next_indent = format!("{}    ", indent);

    writeln!(w, "{}{{", indent)?;
    writeln!(w, "{}  \"level\": {},", indent, level)?;
    writeln!(w, "{}  \"value\": \"{}\",", indent, value)?;

    if level >= max_depth {
        writeln!(w, "{}  \"children\": []", indent)?;
    } else {
        // Branching factor varies 2-3 per node so the tree is not perfectly uniform.
        let children = if (level + sibling_idx).is_multiple_of(2) {
            3
        } else {
            2
        };
        writeln!(w, "{}  \"children\": [", indent)?;
        for c in 0..children {
            write_deep_node(w, level + 1, max_depth, c, &next_indent)?;
            if c + 1 < children {
                writeln!(w, ",")?;
            } else {
                writeln!(w)?;
            }
        }
        writeln!(w, "{}  ]", indent)?;
    }

    write!(w, "{}}}", indent)?;
    Ok(())
}

fn write_keys<W: Write>(w: &mut W, field_count: usize) -> std::io::Result<()> {
    // keys.json shape: single object with N distinct field names.
    // Each entry is an object with type/value to vary types across the cache.
    writeln!(w, "{{")?;
    for i in 0..field_count {
        let key = format!("field_{:06}", i);
        let kind = i % 4;
        let last = i + 1 == field_count;
        match kind {
            0 => writeln!(
                w,
                "  \"{}\": {{ \"type\": \"string\", \"value\": \"v{}\" }}{}",
                key,
                i,
                if last { "" } else { "," }
            )?,
            1 => writeln!(
                w,
                "  \"{}\": {{ \"type\": \"number\", \"value\": {} }}{}",
                key,
                i,
                if last { "" } else { "," }
            )?,
            2 => writeln!(
                w,
                "  \"{}\": {{ \"type\": \"boolean\", \"value\": {} }}{}",
                key,
                i % 2 == 0,
                if last { "" } else { "," }
            )?,
            _ => writeln!(
                w,
                "  \"{}\": {{ \"type\": \"null\", \"value\": null }}{}",
                key,
                if last { "" } else { "," }
            )?,
        }
    }
    writeln!(w, "}}")?;
    Ok(())
}
