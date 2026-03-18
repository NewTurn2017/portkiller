use std::env;
use std::fmt;
use std::process::{self, Command};

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

#[derive(Debug, Clone)]
struct PortInfo {
    port: u16,
    pid: u32,
    command: String,
    user: String,
}

impl fmt::Display for PortInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{:<6} {} (PID {})", self.port, self.command, self.pid)
    }
}

fn scan_ports() -> Vec<PortInfo> {
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("{} lsof 실행 실패: {e}", "✗".red());
            process::exit(1);
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports: Vec<PortInfo> = Vec::new();

    for line in stdout.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 9 {
            continue;
        }

        let command = cols[0].to_string();
        let pid: u32 = match cols[1].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let user = cols[2].to_string();

        // lsof NAME column — "*:3000", "127.0.0.1:8080", "[::1]:5173"
        let name = cols[8];
        let port = parse_port(name);

        if let Some(port) = port {
            ports.push(PortInfo {
                port,
                pid,
                command,
                user,
            });
        }
    }

    ports.sort_by_key(|p| (p.port, p.pid));
    ports.dedup_by_key(|p| (p.port, p.pid));
    ports
}

fn parse_port(name: &str) -> Option<u16> {
    name.rsplit(':').next()?.parse().ok()
}

fn kill_process(info: &PortInfo) {
    let status = Command::new("kill")
        .args(["-9", &info.pid.to_string()])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!(
                "  {} :{} {} (PID {})",
                "✓".green().bold(),
                info.port.to_string().cyan(),
                info.command.dimmed(),
                info.pid.to_string().dimmed(),
            );
        }
        _ => {
            eprintln!(
                "  {} :{} 종료 실패 — sudo 필요할 수 있습니다",
                "✗".red().bold(),
                info.port,
            );
        }
    }
}

fn print_help() {
    println!(
        r#"
{}  개발 서버 포트를 즉시 종료합니다.

{}
  pk              대화형 선택으로 포트 종료
  pk <port>       특정 포트 즉시 종료
  pk -a, --all    모든 리스닝 포트 종료
  pk -l, --list   리스닝 포트 목록만 표시
  pk -h, --help   도움말 표시
"#,
        "pk".cyan().bold(),
        "USAGE".yellow().bold(),
    );
}

fn print_table(ports: &[PortInfo]) {
    let header = format!("  {:<8} {:<8} {:<16} {}", "PORT", "PID", "COMMAND", "USER");
    println!("{}", header.dimmed());
    println!("{}", "  ─".repeat(16).dimmed());

    for p in ports {
        println!(
            "  {:<8} {:<8} {:<16} {}",
            format!(":{}", p.port).cyan(),
            p.pid.to_string().yellow(),
            p.command.white().bold(),
            p.user.dimmed(),
        );
    }
}

fn interactive_kill(ports: &[PortInfo]) {
    let items: Vec<String> = ports
        .iter()
        .map(|p| format!(":{:<6}  {:<14}  PID {}", p.port, p.command, p.pid))
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("종료할 포트를 선택하세요 (Space: 선택, Enter: 확인)")
        .items(&items)
        .interact_opt()
        .unwrap_or_else(|_| {
            println!("\n{}", "취소됨.".dimmed());
            process::exit(0);
        });

    match selections {
        Some(indices) if !indices.is_empty() => {
            println!();
            for &i in &indices {
                kill_process(&ports[i]);
            }
        }
        _ => {
            println!("\n{}", "선택된 포트 없음.".dimmed());
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let ports = scan_ports();
    if ports.is_empty() {
        println!("  {} 리스닝 중인 포트가 없습니다.", "✓".green());
        return;
    }

    if args.iter().any(|a| a == "-l" || a == "--list") {
        println!(
            "\n  {} 개의 리스닝 포트 발견\n",
            ports.len().to_string().cyan().bold()
        );
        print_table(&ports);
        println!();
        return;
    }

    if args.iter().any(|a| a == "-a" || a == "--all") {
        println!(
            "\n  {} {} 개의 포트를 종료합니다\n",
            "⚡".yellow(),
            ports.len()
        );
        for p in &ports {
            kill_process(p);
        }
        return;
    }

    let port_args: Vec<u16> = args.iter().filter_map(|a| a.parse::<u16>().ok()).collect();

    if !port_args.is_empty() {
        for target in &port_args {
            let matches: Vec<&PortInfo> = ports.iter().filter(|p| p.port == *target).collect();

            if matches.is_empty() {
                eprintln!(
                    "  {} :{} 에서 리스닝 중인 프로세스가 없습니다",
                    "✗".red(),
                    target,
                );
            } else {
                for p in matches {
                    kill_process(p);
                }
            }
        }
        return;
    }

    println!(
        "\n  {} 개의 리스닝 포트 발견\n",
        ports.len().to_string().cyan().bold()
    );
    print_table(&ports);
    println!();
    interactive_kill(&ports);
}
