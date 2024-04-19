#!/bin/sh
pdfanno2org . -o output.org
diff output.org output-test.org
