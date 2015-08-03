import subprocess
import glob
from os.path import splitext, exists
import sys

for f in glob.glob("tests/*/*.rs"):
    if exists(splitext(f)[0] + '.csv') and len(sys.argv) > 1:
        continue
    subprocess.Popen('LD_LIBRARY_PATH=/vol/elvis/samgarm/rust/x86_64-unknown-linux-gnu/stage2/lib RUST_FOLDER=/vol/elvis/samgarm/rust/x86_64-unknown-linux-gnu/stage2/lib/rustlib/x86_64-unknown-linux-gnu/lib   rustc -o out -Zsave-analysis ' + f, shell=True).communicate()[0]
    subprocess.Popen('mv dxr-temp/unknown_crate.csv ' + splitext(f)[0] + '.csv', shell=True).communicate()[0]
