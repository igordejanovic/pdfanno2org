use anyhow::{Context, Result};
use chrono::DateTime;
use pathdiff::diff_paths;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use textwrap::wrap;

use clap::Parser;
use ignore::WalkBuilder;
use lopdf::{Document, Object};

#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Cli {
    #[clap(value_name="ROOT DIR", value_hint = clap::ValueHint::DirPath)]
    root: PathBuf,
    #[clap(short, long, value_name="output file", value_hint = clap::ValueHint::FilePath)]
    output: Option<PathBuf>,

    #[clap(long, help = "ignore .gitignore file")]
    no_git_ignore: bool,

    #[clap(long, help = "ignore .ignore file")]
    no_ignore: bool,

    #[clap(long, help = "include hidden files")]
    hidden: bool,
}

// In case of error we'll skip annotation and continue.
macro_rules! skip {
    ($reason:literal) => {
        eprintln!(r"Warning: {}. Skipping annotation.", $reason);
        continue
    };
}

struct Convertor {
    output: File,
    output_path: PathBuf,
}

impl Convertor {
    fn extract_annotations(&mut self, pdf: Document, relative_path: String) -> Result<u32> {
        let mut prev_chapter_title = String::new();
        let mut title_written = false;
        let mut annotation_count = 0;
        for (page_num, page) in pdf.get_pages() {
            for annotation in pdf.get_page_annotations(page) {
                // println!("NEW ANNOTATION");
                // dbg!(&annotation);
                let subtype = annotation
                    .get(b"Subtype")
                    .and_then(Object::as_name_str)
                    .unwrap_or("");
                if subtype != "Highlight" {
                    continue;
                };

                let contents = String::from_utf8_lossy(
                    annotation
                        .get(b"Contents")
                        .and_then(Object::as_str)
                        .unwrap_or(b""),
                )
                .into_owned();
                // onyxtag keeps all the interesting data.
                // we are not supporting highlight created by other means at
                // the moment.
                let Ok(onyxtag) = serde_json::from_slice::<Value>(
                    String::from_utf8_lossy(
                        annotation
                            .get(b"onyxtag")
                            .and_then(Object::as_str)
                            .unwrap_or(b""),
                    )
                    .as_bytes(),
                ) else {
                    skip!("Invalid JSON in onyxtag");
                };

                let Ok(extra_attr) = serde_json::from_str::<Value>(
                    onyxtag.get("extra_attr").unwrap().as_str().unwrap(),
                ) else {
                    skip!("Invalid JSON in onyxtag");
                };

                fn get_str_value<'i>(v: &'i Value, key: &str) -> Option<&'i str> {
                    v.get(key)?.as_str()
                }
                fn get_i64_value(v: &Value, key: &str) -> Option<i64> {
                    v.get(key)?.as_i64()
                }

                let Some(chapter_title) = get_str_value(&extra_attr, "chapterTitle") else {
                    skip!("Cannot extract 'chapterTitle'");
                };

                let Some(quote) = get_str_value(&extra_attr, "quote") else {
                    skip!("Cannot extract 'quote'");
                };
                let quote =
                    wrap(&quote.replace(['\r', '\n'], " ").replace("  ", " "), 100).join("\n");

                let timestamp = DateTime::from_timestamp_millis(
                    get_i64_value(&extra_attr, "createdAt").unwrap_or_default(),
                )
                .unwrap_or_default();

                if !title_written {
                    writeln!(self.output, "* [[pdf:{relative_path}][{relative_path}]]")?;
                    title_written = true;
                }

                if prev_chapter_title != chapter_title && !chapter_title.is_empty() {
                    writeln!(self.output, "** {}", chapter_title)?;
                    prev_chapter_title = chapter_title.into();
                }

                writeln!(
                    self.output,
                    "**{} [[pdf:{relative_path}::{}][{}..., page {}]]",
                    if !chapter_title.is_empty() { "*" } else { "" },
                    page_num,
                    quote.chars().take(50).collect::<String>(),
                    page_num
                )?;
                writeln!(self.output, ":PROPERTIES:")?;
                writeln!(self.output, ":DATETIME: {timestamp}")?;
                writeln!(self.output, ":END:")?;
                writeln!(self.output, "#+begin_quote")?;
                writeln!(self.output, "{quote}")?;
                writeln!(self.output, "#+end_quote\n")?;
                if !contents.is_empty() {
                    writeln!(self.output, "{contents}\n")?;
                }
                annotation_count += 1;
            }
        }
        Ok(annotation_count)
    }

    fn process_files(
        &mut self,
        root: &Path,
        git_ignore: bool,
        ignore: bool,
        hidden: bool,
    ) -> Result<()> {
        let mut files = 0;
        let mut annotations = 0;
        for entry in WalkBuilder::new(root)
            .git_ignore(git_ignore)
            .ignore(ignore)
            .hidden(hidden)
            .build()
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                || !path
                    .extension()
                    .is_some_and(|ext| ext.to_ascii_lowercase() == "pdf")
            {
                continue;
            }
            print!("Processing {:?}", path);
            let document = Document::load(path)
                .with_context(|| format!("Failed to read PDF from: {:?}", path))?;
            let relative_path =
                diff_paths(path, self.output_path.canonicalize()?.parent().unwrap())
                    .unwrap_or(path.to_path_buf());
            let relative_path_str = String::from(relative_path.as_os_str().to_string_lossy());
            let Ok(annotations_for_file) = self.extract_annotations(document, relative_path_str)
            else {
                eprintln!("Error processing file. Skipping");
                println!();
                continue;
            };
            println!(" ... {annotations_for_file} annotation(s)");
            files += 1;
            annotations += annotations_for_file;
        }
        println!("Total files = {files}");
        println!("Total annotations = {annotations}");
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = fs::canonicalize(cli.root)?;

    // get output file
    let output = match cli.output {
        Some(o) => o,
        None => if path.is_dir() {
            Some(path.as_path())
        } else {
            path.parent()
        }
        .unwrap_or(Path::new("."))
        .join(Path::new("output.org"))
        .with_extension("org"),
    };

    let mut convertor = Convertor {
        output: File::create(output.clone())?,
        output_path: output.clone(),
    };
    convertor.process_files(&path, !cli.no_git_ignore, !cli.no_ignore, cli.hidden)?;
    println!("Output: {:?}", output.canonicalize()?);
    Ok(())
}
