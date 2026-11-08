use std::io::{self, BufWriter, Cursor, Read, Write};
use std::process::{Command, Stdio};

use bytelines::ByteLinesReader;
use regex::bytes::Regex;

use crate::config::Config;

pub fn render(input: impl Read, output: &mut dyn Write, config: &Config) -> io::Result<()> {
    let input = prepare_input(input)?;
    let mut output = GraphColors::new(output);
    crate::delta::delta(Cursor::new(input).byte_lines(), &mut output, config)?;
    output.finish()
}

fn prepare_input(mut input: impl Read) -> io::Result<Vec<u8>> {
    let mut raw = Vec::new();
    input.read_to_end(&mut raw)?;
    let sgr = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let clean = sgr.replace_all(&raw, &b""[..]);
    let summary = Regex::new(r"(?m)^[[:space:]]*[0-9]+ files? changed(,|$)").unwrap();
    let stat = if !raw.is_empty() && !summary.is_match(&clean) {
        diff_stat(&clean)
    } else {
        None
    };
    let mut prepared = Vec::with_capacity(raw.len());
    if let Some(stat) = stat {
        prepared.push(b'\n');
        prepared.extend_from_slice(&stat);
    }
    for byte in raw {
        if byte == b'\t' {
            prepared.extend_from_slice("├──┤".as_bytes());
        } else {
            prepared.push(byte);
        }
    }
    Ok(prepared)
}

fn diff_stat(input: &[u8]) -> Option<Vec<u8>> {
    let mut child = Command::new("git")
        .args(["apply", "--stat"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdin = child.stdin.take()?;
    // Drain output while supplying input so neither pipe can block the other.
    let result = std::thread::scope(|scope| {
        scope.spawn(move || {
            let _ = stdin.write_all(input);
        });
        child.wait_with_output()
    })
    .ok()?;
    (result.status.success() && !result.stdout.is_empty()).then_some(result.stdout)
}

struct GraphColors<W: Write> {
    output: BufWriter<W>,
    line: Vec<u8>,
    graph: Regex,
}

impl<W: Write> GraphColors<W> {
    fn new(output: W) -> Self {
        Self {
            output: BufWriter::with_capacity(4096, output),
            line: Vec::new(),
            graph: Regex::new(r"\|[[:space:]]+[0-9]+[[:space:]]+[+-]+$").unwrap(),
        }
    }

    fn emit_line(&mut self) -> io::Result<()> {
        if let Some(graph) = self.graph.find(&self.line) {
            self.output.write_all(&self.line[..graph.start()])?;
            let mut offset = graph.start();
            while offset < self.line.len() {
                let byte = self.line[offset];
                if byte == b'+' || byte == b'-' {
                    let end = offset
                        + self.line[offset..]
                            .iter()
                            .take_while(|&&value| value == byte)
                            .count();
                    self.output.write_all(if byte == b'+' {
                        b"\x1b[32m"
                    } else {
                        b"\x1b[31m"
                    })?;
                    self.output.write_all(&self.line[offset..end])?;
                    self.output.write_all(b"\x1b[m")?;
                    offset = end;
                } else {
                    self.output.write_all(&[byte])?;
                    offset += 1;
                }
            }
        } else {
            self.output.write_all(&self.line)?;
        }
        self.output.write_all(b"\n")?;
        self.line.clear();
        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        if !self.line.is_empty() {
            self.emit_line()?;
        }
        self.output.flush()
    }
}

impl<W: Write> Write for GraphColors<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for part in bytes.split_inclusive(|&byte| byte == b'\n') {
            self.line.extend_from_slice(part);
            if self.line.last() == Some(&b'\n') {
                self.line.pop();
                self.emit_line()?;
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATCH: &[u8] =
        b"diff --git a/file b/file\n--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new\n";

    #[test]
    fn missing_stats_are_generated_without_changing_patch_bytes() {
        let prepared = prepare_input(PATCH).unwrap();
        assert!(prepared.starts_with(b"\n file |"));
        assert!(prepared.ends_with(PATCH));
        assert!(String::from_utf8_lossy(&prepared)
            .contains("1 file changed, 1 insertion(+), 1 deletion(-)"));
    }

    #[test]
    fn existing_colored_stats_are_retained_and_tabs_are_marked() {
        let input = b"\x1b[32m 1 file changed, 1 insertion(+)\x1b[m\n\tcontent\n";
        let expected = "\x1b[32m 1 file changed, 1 insertion(+)\x1b[m\n├──┤content\n";
        assert_eq!(prepare_input(&input[..]).unwrap(), expected.as_bytes());
        assert!(prepare_input(&b""[..]).unwrap().is_empty());
        assert_eq!(prepare_input(&b"\xff\n"[..]).unwrap(), b"\xff\n");
    }

    #[test]
    fn graph_coloring_is_independent_of_write_boundaries() {
        let input = b"\x1b]1717;1\x07\n file | 8 +++--++-\n\x1b]8;;lazygit-edit://file:1\x07file\x1b]8;;\x07\nplain";
        let expected = b"\x1b]1717;1\x07\n file | 8 \x1b[32m+++\x1b[m\x1b[31m--\x1b[m\x1b[32m++\x1b[m\x1b[31m-\x1b[m\n\x1b]8;;lazygit-edit://file:1\x07file\x1b]8;;\x07\nplain\n";
        for size in 1..=input.len() {
            let mut actual = Vec::new();
            let mut writer = GraphColors::new(&mut actual);
            for chunk in input.chunks(size) {
                writer.write_all(chunk).unwrap();
            }
            writer.finish().unwrap();
            drop(writer);
            assert_eq!(actual, expected, "chunk size {size}");
        }
    }
}
