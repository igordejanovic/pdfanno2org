# pdfanno2org

Extracts PDF annotations to Org Mode file. Currently supports only [Onyx Boox
NeoReader](https://onyxboox.com/) highlights as NeoReader conveniently extract
the text under the highlight which seems to be not that straightforward to do
otherwise.

# Why would you need this?

If you are reading a lot of PDFs and highlight/write notes directly into a pdf,
and at the same time you can't live without Emacs Org Mode you would want to
have notes from all your pdfs always up-to-date and available inside an Org Mode
file.

What I find convenient is to use browsers print-to-pdf feature when I find
something interesting to read on the web. These pdfs go to a folder which is
synced with my Boox device using [SyncThing](https://github.com/syncthing/).
Similarly, all books and papers are synced too. pdfanno2org helps me to extract
all annotations in one go.

# Features

- Operates on a single file or recursively on a given folder. Creates a single
  output Org Mode file in each case with all annotations found.
- Create Org Mode title which links to the given pdf for each pdf file that has
  highlight annotations .
- Create sub-title for each annotation. The title links to the annotation page.
  The title section contains highlighted content as a Org Mode quote block and
  any additional text which is embedded inside the annotation.
- If there is a chapter title recognized by NeoReader it will be an intermediate
  sub-title.
- Fast! Written in Rust using lopdf. In initial tests on my machine it processes
  ~100 pdfs per second. Of course, pdfs/sec will depend on the size of pdfs,
  number of annotations, machine power etc.

# Installation

[Install Rust](https://www.rust-lang.org/tools/install) and then:

``` sh
cargo install --git https://github.com/igordejanovic/pdfanno2org.git
```

# Usage

``` sh
pdfanno2org /path/to/pdf/files

# Optionaly you can provide output file name
# If not given it will be created from the last part of the input path
pdfanno2org /path/to/pdf/files -o notes.org
```

# TODO

- [ ] Use CUSTOM_ID Org Mode feature with id created from PDFs meta-data which
      should not change on adding annotation, renaming/moving pdfs around to
      enable stable links to generated notes.
- [ ] Support for handwritten notes. Not sure how to handle this. I'll probably
      just make a link to the page where handwritten notes are found. Maybe the
      best approach would be to use NeoReader OCR + finding/linking to
      recognized text boxes.
- [ ] Extract the text under the highlight, thus supporting any
      reader/highlighter tool. I won't work on this as I use Boox only at the
      moment so for my use-case this suffice, but if anyone wants to give it a
      try I would be happy to review/merge.
- [ ] Do not recreate document from scratch but just update to keep any manually
      added content. This will require parsing Org file and figuring out what
      should be updated. CUSTOM_ID based on pdf meta-data should be a convenient
      way to find Org Mode title even if it has beed moved around.

# Credits

- https://github.com/J-F-Liu/lopdf
- https://github.com/jeertmans/rpdf - not used directly but I first found this
  project and examined to see patterns of using lopdf.
- Several other helpful libraries without which it would be much harder to
  implement this tool. See Cargo.toml.
