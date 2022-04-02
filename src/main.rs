use std::{cmp, collections::HashSet, env, fs::File, io::Write, ops::Range, path::PathBuf};

use clap::StructOpt;
use dotenv::dotenv;
use futures::stream::FuturesUnordered;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use nipper::Document;
use reqwest::Client;

const ALL_YEARS: Range<usize> = 2000..2022;
const DEFAULT_START: usize = 2017;

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(parse(from_os_str))]
    dest: PathBuf,

    /// All papers from given year to present. 
    /// Defaults to 2017.
    #[clap(long, parse(try_from_str=parse_year_opt))]
    papers_from: Option<usize>,

    /// Year group to download pastpapers for.
    #[clap(short, long, arg_enum)]
    year_group: YearGroup
}

#[derive(Debug, Clone, Copy, clap::ArgEnum)]
enum YearGroup {
    Y1,
    Y2
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args = Args::parse();
    let username =
        env::var("DOC_USERNAME").expect("Couldn't find DOC_USERNAME environment variable");
    let password =
        env::var("DOC_PASSWORD").expect("Couldn't find DOC_PASSWORD environment variable");
    let client = reqwest::Client::builder().cookie_store(true).build()?;

    let start = args.papers_from.unwrap_or(DEFAULT_START);
    let years = start..ALL_YEARS.end;
    let year_group = args.year_group;

    // Indicatif setup
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    let mpb = MultiProgress::new();

    let mut futs = FuturesUnordered::new();
    for year in years {
        // Setup progress bar for year
        let pb = mpb.add(ProgressBar::new(1000));
        let academic_year = year_to_string(year);
        pb.set_style(spinner_style.clone());
        pb.set_prefix(&format!("[{}|?/?]", &academic_year.as_str()));
        pb.set_message("waiting...");

        // Clone args to move to future
        let client = client.clone();
        let dest = args.dest.clone();
        let uname = username.clone();
        let pword = password.clone();

        let task = tokio::spawn(async move {
            let url = year_to_url(year);
            let prefix = year_to_prefix(year, year_group);
            let resp = client
                .get(&url)
                .basic_auth(&uname, Some(&pword))
                .send()
                .await
                .map_err(|e| format!("{e}"))?
                .text()
                .await
                .map_err(|e| format!("{e}"))?;

            let urls = Document::from(&resp)
                .select("p")
                .select("a")
                .iter()
                .filter_map(|link| {
                    let href = link.attr("href").unwrap();
                    if href.starts_with(prefix) {
                        Some(url.clone() + &href)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();

            let academic_year = year_to_string(year);
            let year_dest = dest.clone().join(academic_year.as_str());
            let total = urls.len();
            for (i, url) in urls.iter().enumerate() {
                pb.set_prefix(&format!(
                    "[{}|{:0>2}/{:0>2}]",
                    &academic_year.as_str(),
                    i + 1,
                    total
                ));
                download_and_save(&client, &uname, &pword, url, year_dest.clone(), &pb).await?;
                pb.reset();
            }

            pb.finish_with_message("done ✨");

            Ok::<(), String>(())
        });

        futs.push(task);
    }

    let handle_m = tokio::task::spawn_blocking(move || mpb.join().unwrap());
    while let Some(result) = futs.next().await {
        result??;
    }
    handle_m.await?;

    Ok(())
}

pub async fn download_and_save(
    client: &Client,
    username: &str,
    password: &str,
    url: &str,
    dir: PathBuf,
    pb: &ProgressBar,
) -> Result<(), String> {
    // Reqwest setup
    let resp = client
        .get(url)
        .basic_auth(username, Some(password))
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = resp
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    // Prepare the destination file
    std::fs::create_dir_all(&dir).or(Err(format!("Failed to create destination directory")))?;
    let fname = url.rsplit_once("/").unwrap().1;
    let path = &dir.join(fname);
    let mut file = File::create(path).or(Err(format!(
        "Failed to create file '{}'",
        path.to_string_lossy()
    )))?;

    // Download chunks
    pb.set_length(total_size);
    pb.set_message(&format!("downloading...: {fname}"));
    let mut downloaded = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    Ok(())
}

fn year_to_url(year: usize) -> String {
    let year_str = year_to_string(year);
    format!("https://exams.doc.ic.ac.uk/pastpapers/papers.{year_str}/")
}

fn year_to_string(year: usize) -> String {
    let year = year % 100;
    let prev = if year == 0 { 99 } else { year - 1 };
    format!("{prev:0>2}-{year:0>2}")
}

fn year_to_prefix(year: usize, year_group: YearGroup) -> &'static str {

    let (old, new) = match year_group {
        YearGroup::Y1 => ("C1", "COMP4"),
        YearGroup::Y2 => ("C2", "COMP5")
    };

    match year {
        2000..=2020 => old,
        _ => new,
    }
}

fn parse_year_opt(s: &str) -> Result<usize, String> {
    let year: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a calendar year"))?;
    if ALL_YEARS.contains(&year) {
        Ok(year)
    } else {
        Err(format!(
            "DoC papers are not available for {}. Available years are from {} to {}.",
            year, ALL_YEARS.start, ALL_YEARS.end
        ))
    }
}
