import sys

INPUT = sys.argv[1]
OUTPUT = sys.argv[2]

BEGIN_TANGLE = "#+begin_src"
TANGLE_LANGS = ["emacs-lisp", "elisp"]
CANCEL_TANGLE = ":tangle no"
END_TANGLE = "#+end_src"


def normalize(line):
    return line.strip().lower()


def line_begins_tangling(line_norm):
    return (
        line_norm.startswith(BEGIN_TANGLE)
        and any([lang in line_norm for lang in TANGLE_LANGS])
        and not CANCEL_TANGLE in line_norm
    )


def line_ends_tangling(line_norm):
    return line_norm == END_TANGLE


with open(INPUT, "r") as input_file:
    with open(OUTPUT, "w") as output_file:
        tangle = False
        for line in input_file.readlines():
            line_norm = normalize(line)
            if tangle:
                if line_ends_tangling(line_norm):
                    tangle = False
                else:
                    output_file.write(line)
            elif line_begins_tangling(line_norm):
                tangle = True
