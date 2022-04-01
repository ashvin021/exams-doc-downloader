# Exams DoC Downloader

A cli tool to download Y2 past papers from [exams.doc.ic.ac.uk](exams.doc.ic.ac.uk). An excercise in procrastination, written in Rust 🦀.

## Usage

Clone the repo, and run the following substituting your credentials:

```shell
$ cd exams-doc-downloader
$ echo "DOC_USERNAME=<your-username-here>" >> .env
$ echo "DOC_PASSWORD=<your-password-here>" >> .env
```

Then you can run the tool with `cargo`, using the following:

```shell
$ cargo run --release -- [OPTIONS] <DEST>
```

**Options:**

```shell
  -h, --help                         Print help information
      --papers-from <PAPERS_FROM>    All papers from this year to present. Defaults to 2017.
```

## Possible Future Stuff

- [ ] Allow downloading from different / multiple years.
- [ ] Add a release binary.
