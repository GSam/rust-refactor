import subprocess
import glob
from os.path import splitext

for f in glob.glob("tests/*/*.rs"):
    subprocess.Popen('LD_LIBRARY_PATH=/vol/elvis/samgarm/rust/x86_64-unknown-linux-gnu/stage2/lib RUST_FOLDER=/vol/elvis/samgarm/rust/x86_64-unknown-linux-gnu/stage2/lib/rustlib/x86_64-unknown-linux-gnu/lib   rustc -Zsave-analysis ' + f, shell=True).communicate()[0]
    subprocess.Popen('mv dxr-temp/unknown_crate.csv ' + splitext(f)[0] + '.csv', shell=True).communicate()[0]
