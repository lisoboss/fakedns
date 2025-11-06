use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

fn reverse_domain(domain: &str) -> String {
    let mut parts: Vec<&str> = domain.split('.').collect();
    parts.reverse();
    parts.join(".")
}

fn is_subdomain(sub: &str, domain: &str) -> bool {
    reverse_domain(sub).starts_with(&reverse_domain(domain))
}

fn filter_domains(domains: Vec<String>) -> Vec<String> {
    use std::collections::HashMap;

    let mut domains_dict: HashMap<String, Vec<String>> = HashMap::new();

    for domain in domains {
        if domain.is_empty() || domain.matches('.').count() < 1 {
            continue;
        }

        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            continue;
        }
        let domain_p = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);

        let entry = domains_dict.entry(domain_p.clone()).or_default();
        let mut need_add = true;
        let mut to_remove = None;

        for saved in entry.iter() {
            if is_subdomain(&domain, saved) {
                need_add = false;
                break;
            }
            if is_subdomain(saved, &domain) {
                to_remove = Some(saved.clone());
                break;
            }
        }

        if let Some(rm) = to_remove {
            entry.retain(|d| d != &rm);
        }

        if need_add {
            entry.push(domain);
        }
    }

    let mut result = Vec::new();
    for v in domains_dict.values() {
        result.extend_from_slice(v);
    }
    result
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("用法: build_domains -i <input1> [input2 ...] -o <output>");
        std::process::exit(1);
    }

    let mut inputs: Vec<PathBuf> = Vec::new();
    let mut output: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => {
                i += 1;
                while i < args.len() && args[i] != "-o" {
                    inputs.push(PathBuf::from(&args[i]));
                    i += 1;
                }
                continue;
            }
            "-o" => {
                i += 1;
                if i < args.len() {
                    output = Some(PathBuf::from(&args[i]));
                }
            }
            _ => {}
        }
        i += 1;
    }

    let output = output.expect("必须指定输出文件路径");

    // 读取所有输入文件
    let mut all_domains = Vec::new();
    for path in &inputs {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with("regexp:") {
                continue;
            }
            let line = line
                .strip_prefix("full:")
                .unwrap_or(&line)
                .trim()
                .to_string();
            if !line.is_empty() {
                all_domains.push(line);
            }
        }
    }

    // 去重
    all_domains.sort();
    all_domains.dedup();

    // 过滤子域
    let filtered = filter_domains(all_domains);

    // 写入输出文件
    let mut f = File::create(&output)?;
    for d in filtered {
        writeln!(f, "{}", d)?;
    }

    Ok(())
}
