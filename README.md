# ns_zombies
Analyzing Forest's zday individual stats.
To run, simply use the command `cargo run --release` (rust must be installed).
The script looks for `happenings.xml` first, filters the non-zday events out, 
and then writes the filtered results to `activities.xml`. If `happenings.xml` 
doesn't exist, the script will attempt to read from `activities.xml`, aborting if it doesn't
exist. The results are written to `zdata.csv`. I know the code isn't documented well (or at all), and I may or may not get around to doing that. I will also likely add more stats at some point. This repository is licensed with the MIT License, see `license.md` for details. In practice, this means that you can do whatever you want, so long as you give me credit for
the original code.