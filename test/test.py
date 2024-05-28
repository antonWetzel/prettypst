# update the test results

styles = ["default", "otbs"]

import os
import subprocess
import shutil


test_directory = os.path.dirname(__file__)
project_directory = os.path.dirname(test_directory)

source_directory = os.path.join(test_directory, "source")
os.chdir(project_directory)

for style in styles:
    out_directory = os.path.join(test_directory, style)
    shutil.rmtree(out_directory)
    os.mkdir(out_directory)

    for entry in os.scandir(source_directory):
        if entry.is_dir():
            print("todo: support folders")
            continue
        source_path = entry.path
        out_path = entry.path.replace("source", style)
        subprocess.run(["cargo", "run", "--", f"--style={style}", source_path, f"--output={out_path}"], stderr=subprocess.DEVNULL)
        source_path = os.path.relpath(source_path, test_directory)
        out_path = os.path.relpath(out_path, test_directory)
        print(f"formatted {source_path:<30} to {out_path:<30}")
