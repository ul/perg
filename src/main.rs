extern crate rayon;
extern crate regex;

use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

fn main() {
    // Vim grep entry is path:line:column:text, indexing is 1-based.
    let entry_re = Regex::new(r"^(.+):(\d+):\d+:(.*)$").unwrap();

    // First build a map of all changes to batch work to be done in the same file and to parallelize
    // work over different files (as an alternative to streaming solution of making changes for
    // every matching line).

    let mut changes: HashMap<String, Vec<(usize, String)>> = HashMap::default();
    for entry in io::stdin().lock().lines().filter_map(|x| x.ok()) {
        if let Some(caps) = entry_re.captures(&entry) {
            let path = caps[1].to_string();
            let line: usize = caps[2].parse().unwrap();
            let text = &caps[3];
            changes
                .entry(path)
                .and_modify(|v| v.push((line, text.to_string())))
                .or_insert_with(|| vec![(line, text.to_string())]);
        }
    }

    // TODO proper error reporting
    changes.into_par_iter().for_each(|(path, changes)| {
        let mut lines = BufReader::new(File::open(&path).unwrap())
            .lines()
            .map(|x| x.unwrap().to_string())
            .collect::<Vec<_>>();
        for (line_num, line) in changes {
            // NOTE line_num is 1-based, but vector index is 0-based
            lines[line_num - 1] = line;
        }
        let mut output = BufWriter::new(File::create(&path).unwrap());
        for line in lines {
            write!(output, "{}\n", line).unwrap();
        }
    })
}
