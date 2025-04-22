# update the test results

styles = ["default", "otbs"]

import os
import subprocess
import shutil
import sys

if len(sys.argv) >= 2:
    include = sys.argv[1]
else:
    include = None

test_directory = os.path.dirname(__file__)
project_directory = os.path.dirname(test_directory)

source_directory = os.path.join(test_directory, "source")
os.chdir(project_directory)

for style in styles:
    out_directory = os.path.join(test_directory, style)
    shutil.rmtree(out_directory, ignore_errors=True)
    for (root, dirs, files) in os.walk(source_directory):
        source = "source"
        idx = root.rfind(source)
        out_root = root[:idx] + style + root[idx + len(source):]
        os.mkdir(out_root)
        for file in files:
            source_path = root + "/" + file
            if include != None and include not in source_path:
                continue
            out_path = out_root + "/" + file
            subprocess.run(["cargo", "run", "--", f"--style={style}", source_path, f"--output={out_path}"], stderr=subprocess.DEVNULL)
            source_path = os.path.relpath(source_path, test_directory)
            out_path = os.path.relpath(out_path, test_directory)
            print(f"formatted {source_path:<30} to {out_path:<30}")
