use std::{io, process::Command};

type Parser = fn(&str) -> Vec<Listener>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listener {
    pub protocol: String,
    pub address: String,
    pub port: u16,
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub exposure: Exposure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exposure {
    Local,
    Lan,
    All,
}

impl Exposure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Lan => "network",
            Self::All => "all-interfaces",
        }
    }
}

pub fn classify(address: &str) -> Exposure {
    let clean = address.trim_matches(['[', ']']);
    if matches!(clean, "127.0.0.1" | "::1" | "localhost") || clean.starts_with("127.") {
        Exposure::Local
    } else if matches!(clean, "0.0.0.0" | "::" | "*" | "[::]") {
        Exposure::All
    } else {
        Exposure::Lan
    }
}

pub fn parse_endpoint(value: &str) -> Option<(String, u16)> {
    let value = value.trim();
    let (address, port) = value.rsplit_once(':')?;
    let port = port.trim_matches(['*', '[', ']']).parse().ok()?;
    Some((address.trim_matches(['[', ']']).to_string(), port))
}

pub fn parse_windows_netstat(input: &str) -> Vec<Listener> {
    let mut listeners = Vec::new();
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 5
            || !fields[0].eq_ignore_ascii_case("TCP")
            || !fields[3].eq_ignore_ascii_case("LISTENING")
        {
            continue;
        }
        if let Some((address, port)) = parse_endpoint(fields[1]) {
            listeners.push(Listener {
                protocol: "tcp".into(),
                exposure: classify(&address),
                address,
                port,
                pid: fields[4].parse().ok(),
                process: None,
            });
        }
    }
    listeners
}

pub fn parse_linux_ss(input: &str) -> Vec<Listener> {
    let mut listeners = Vec::new();
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 4 || !fields[0].eq_ignore_ascii_case("LISTEN") {
            continue;
        }
        let Some((address, port)) = parse_endpoint(fields[3]) else {
            continue;
        };
        let tail = fields.get(5..).unwrap_or_default().join(" ");
        let pid = tail
            .find("pid=")
            .and_then(|start| {
                tail[start + 4..]
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
            })
            .and_then(|value| value.parse().ok());
        let process = tail
            .find("((\"")
            .and_then(|start| tail[start + 3..].split('"').next())
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        listeners.push(Listener {
            protocol: "tcp".into(),
            exposure: classify(&address),
            address,
            port,
            pid,
            process,
        });
    }
    listeners
}

pub fn parse_macos_lsof(input: &str) -> Vec<Listener> {
    let mut listeners = Vec::new();
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 9 || fields[0] == "COMMAND" || !line.contains("(LISTEN)") {
            continue;
        }
        let endpoint = fields
            .iter()
            .rev()
            .find(|field| field.contains(':') && **field != "(LISTEN)");
        let Some((address, port)) = endpoint.and_then(|value| parse_endpoint(value)) else {
            continue;
        };
        listeners.push(Listener {
            protocol: "tcp".into(),
            exposure: classify(&address),
            address,
            port,
            pid: fields[1].parse().ok(),
            process: Some(fields[0].to_string()),
        });
    }
    listeners
}

pub fn discover() -> io::Result<Vec<Listener>> {
    #[cfg(target_os = "windows")]
    let (program, args, parser): (&str, &[&str], Parser) =
        ("netstat", &["-ano", "-p", "tcp"], parse_windows_netstat);
    #[cfg(target_os = "linux")]
    let (program, args, parser): (&str, &[&str], Parser) = ("ss", &["-lntpH"], parse_linux_ss);
    #[cfg(target_os = "macos")]
    let (program, args, parser): (&str, &[&str], Parser) =
        ("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"], parse_macos_lsof);
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "unsupported operating system",
    ));

    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{program} exited with {}",
            output.status
        )));
    }
    let mut listeners = parser(&String::from_utf8_lossy(&output.stdout));
    listeners.sort_by_key(|item| (item.port, item.address.clone()));
    listeners.dedup_by(|a, b| a.port == b.port && a.address == b.address && a.pid == b.pid);
    Ok(listeners)
}

pub fn render_table(listeners: &[Listener]) -> String {
    let mut output = format!(
        "{:<6} {:<24} {:<7} {:<9} {:<15} {}\n",
        "PROTO", "ADDRESS", "PORT", "PID", "EXPOSURE", "PROCESS"
    );
    for item in listeners {
        output.push_str(&format!(
            "{:<6} {:<24} {:<7} {:<9} {:<15} {}\n",
            item.protocol,
            item.address,
            item.port,
            item.pid
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".into()),
            item.exposure.as_str(),
            item.process.as_deref().unwrap_or("-")
        ));
    }
    output.push_str(&format!(
        "\n{} listening endpoints; {} exposed on all interfaces\n",
        listeners.len(),
        listeners
            .iter()
            .filter(|item| item.exposure == Exposure::All)
            .count()
    ));
    output
}

pub fn render_json(listeners: &[Listener]) -> String {
    let items = listeners.iter().map(|item| format!("{{\"protocol\":\"{}\",\"address\":\"{}\",\"port\":{},\"pid\":{},\"process\":{},\"exposure\":\"{}\"}}", escape(&item.protocol), escape(&item.address), item.port, item.pid.map(|v| v.to_string()).unwrap_or_else(|| "null".into()), item.process.as_ref().map(|v| format!("\"{}\"", escape(v))).unwrap_or_else(|| "null".into()), item.exposure.as_str())).collect::<Vec<_>>().join(",");
    format!("[{items}]")
}

pub fn render_mermaid(listeners: &[Listener]) -> String {
    let mut output = String::from("flowchart LR\n  client((network))\n  host[local host]\n");
    for (index, item) in listeners.iter().enumerate() {
        let label = format!(
            "{}:{}\\n{}",
            item.address,
            item.port,
            item.process.as_deref().unwrap_or("unknown")
        );
        output.push_str(&format!(
            "  p{index}[\"{}\"]\n  host --> p{index}\n",
            mermaid_escape(&label)
        ));
        if item.exposure != Exposure::Local {
            output.push_str(&format!("  client --> p{index}\n"));
        }
    }
    output
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
fn mermaid_escape(value: &str) -> String {
    value.replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_netstat() {
        let found = parse_windows_netstat("  TCP    0.0.0.0:8080    0.0.0.0:0    LISTENING    1234\n  TCP    127.0.0.1:9000  0.0.0.0:0 LISTENING 42");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].port, found[0].pid), (8080, Some(1234)));
        assert_eq!(found[0].exposure, Exposure::All);
        assert_eq!(found[1].exposure, Exposure::Local);
    }

    #[test]
    fn parses_linux_ss_with_process() {
        let found = parse_linux_ss(
            "LISTEN 0 4096 127.0.0.1:3000 0.0.0.0:* users:((\"python\",pid=777,fd=3))",
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].process.as_deref(), Some("python"));
        assert_eq!(found[0].pid, Some(777));
    }

    #[test]
    fn parses_macos_lsof() {
        let found = parse_macos_lsof("COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME\nnode 321 me 23u IPv6 0x0 0t0 TCP *:5173 (LISTEN)");
        assert_eq!(
            (found[0].port, found[0].process.as_deref()),
            (5173, Some("node"))
        );
    }

    #[test]
    fn parses_ipv6_endpoint() {
        assert_eq!(parse_endpoint("[::1]:8080"), Some(("::1".into(), 8080)));
    }

    #[test]
    fn renders_automation_and_diagram_formats() {
        let item = Listener {
            protocol: "tcp".into(),
            address: "0.0.0.0".into(),
            port: 80,
            pid: Some(1),
            process: Some("web".into()),
            exposure: Exposure::All,
        };
        assert!(render_json(std::slice::from_ref(&item)).contains("\"port\":80"));
        assert!(render_mermaid(&[item]).contains("client --> p0"));
    }
}
