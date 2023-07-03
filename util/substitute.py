import sys, re, json

INPUT = sys.argv[1]
SUBSTITUTIONS = sys.argv[2]
OUTPUT = sys.argv[3]
PATTERN = re.compile(r"(^[^;]*?)\(\s*getnix\s+\"(.*?)\"\s*\)", re.MULTILINE)
# PATTERN = re.compile(r"\(\s*getnix\s+\"(.*?)\"\s*\)", re.MULTILINE)

def toElisp(value):
    if type(value) is bool:
        return ("t" if value else "nil")
    elif type(value) is str and value.startswith("'"):
        return value
    return json.dumps(value)

with open(SUBSTITUTIONS, "r") as substitutions_file:
    substitutions = {
        key: toElisp(value)
        for key, value
        in json.load(substitutions_file).items()
    }

def escape(s):
    return s.replace("\\", "\\\\").replace('"', r'\"')

def substitute(match):
    # key = match.group(1)
    before = match.group(1)
    key = match.group(2)
    if key in substitutions:
        return before + substitutions.get(key)
    else:
        valid_substitutions = "\n".join([
            f"- {key}: {escape(substitutions.get(key))}" for key in substitutions.keys()
        ])
        return (
            f'(error "Invalid nix substitution: {key}\n'
            f'Valid subtitutions:\n\n{valid_substitutions}\n")'
        )

with open(INPUT, "r") as input_file:
    with open(OUTPUT, "w") as output_file:
        output = PATTERN.sub(substitute, input_file.read())
        output_file.write(output)
