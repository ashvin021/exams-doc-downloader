# Exams DoC Downloader

A cli tool to download past papers from [exams.doc.ic.ac.uk](exams.doc.ic.ac.uk). An excercise in procrastination, written in Rust 🦀.

This currently supports Y1 & Y2 papers.

## Usage

Clone the repo, and run the following substituting your credentials:

```shell
$ cd exams-doc-downloader
$ echo "DOC_USERNAME=<your-username-here>" >> .env
$ echo "DOC_PASSWORD=<your-password-here>" >> .env
```

Then you can run the tool with `cargo`, using the following:

```shell
$ cargo run --release -- [OPTIONS] -y <YEAR_GROUP> <DEST>
```

**Options:**

```shell
  -h, --help                         Print help information
      --papers-from <PAPERS_FROM>    All papers from this year to present. Defaults to 2017.
  -y, --year-group <YEAR_GROUP>      Year group to download pastpapers for
                                     [possible values: y1, y2]
```

## Possible Future Stuff

- [ ] Allow downloading more / multiple years.
- [ ] Add a release binary.
