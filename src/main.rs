
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process;
use std::time::Instant;

use getopts::Options;
use regex::Regex;
use walkdir::WalkDir;

type WalkDirEntryVec = Vec<walkdir::DirEntry>;
type WalkDirEntryRVec<'a> = Vec<&'a walkdir::DirEntry>;
type FileNameGrouping<'a> = Vec<(String, u64, WalkDirEntryRVec<'a>)>;
type FileNameMapping<'a> = HashMap<String, WalkDirEntryRVec<'a>>;


//-------------------------------------------------------------------------------------------------
/*

//  https://users.rust-lang.org/t/rusts-equivalent-of-cs-system-pause/4494/4
use std::io;
use std::io::prelude::*;

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "\n\nPress any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
*/

//-------------------------------------------------------------------------------------------------
fn print_usage(program: &str, opts: Options) -> Option<()> {

    let path = Path::new(program);
    let filename = path.file_name()?.to_str()?;
    
    let brief = format!("Usage: {} [options]", filename);
    
    println!("Author: Sarang Baheti, c 2021");
    println!("Source: https://github.com/sarangbaheti/lsdups-rust");
    print!("{}", opts.usage(&brief));

    None
}

//-------------------------------------------------------------------------------------------------
fn get_options(args: &Vec<String>) -> (String, String, String, u64, bool) {

    let mut opts = Options::new();
    opts.optopt("d", "dir", "directory to traverse, defaults to current directory", "<DIRECTORY-PATH>");
    opts.optopt("p", "pattern", "pattern for files, defaults to all files", "<PATTERN>");
    opts.optopt("", "filter", "pattern for files to filter out/skip, defaults to empty-string", "<SKIP-PATTERN>");
    opts.optopt("", "size", "filter all data before this size, defaults to 0", "<unsigned int>");    
    opts.optflag("v", "verbose",  "version information and exit");
    opts.optflag("h", "help",  "prints help");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { 
            println!("{}", f);
            process::exit(0x0100);
        }
    };

    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        process::exit(0x0);
    }

    let verbose = if matches.opt_present("v") { true} else {false};

    let dir2walk = match matches.opt_str("d") {
        Some(s) => s,
        None => ".".to_string(),
    };

    let pattern = match matches.opt_str("p") {
        Some(s) => s,
        None => ".*".to_string(),
    };

    let skip_pattern = match matches.opt_str("filter") {
        Some(s) => s,
        None    => "".to_string(),
    };

    let size_filter = match matches.opt_str("size") {
        Some(s) => s.parse::<u64>().unwrap(),
        None    => 0
    };

    return (dir2walk, pattern, skip_pattern, size_filter, verbose)
}


//-------------------------------------------------------------------------------------------------
fn to_mb(numbytes : u64) -> f64 {
    (numbytes as f64) / 1024.0 / 1024.0
}

//-------------------------------------------------------------------------------------------------
fn dirent_get_size(ent : &walkdir::DirEntry) -> u64 {
    ent.metadata().unwrap().len()
}

//-------------------------------------------------------------------------------------------------
fn dirent_get_size_mb(ent : &walkdir::DirEntry) -> f64 {
    to_mb(dirent_get_size(ent))
}

//-------------------------------------------------------------------------------------------------
fn compare_direntry(a : &walkdir::DirEntry, b : &walkdir::DirEntry) -> std::cmp::Ordering {
    dirent_get_size(b).cmp(&dirent_get_size(&a))
}

//-------------------------------------------------------------------------------------------------
fn is_filename_a_match(e : &walkdir::DirEntry, re : &Regex) -> bool {
    re.is_match(&e.file_name().to_string_lossy())
}

//-------------------------------------------------------------------------------------------------
fn get_filename_grouping(files : &WalkDirEntryVec) -> FileNameGrouping {
    
    //  WalkDirEntryVec -> FileNameMapping -> FileNameGrouping
    //      FileNameMapping  -> helps split and group vector in smaller vectors by filename
    //      FileNameGrouping -> helps capture this information in sorted manner

    //  a very interesting take on grouping
    //  https://hoverbear.org/blog/a-journey-into-iterators/
    let mapping : FileNameMapping 
                = files.iter()
                    .map(|e| {
                        let fname = e.file_name().to_string_lossy().to_string();
                        (fname, e) 
                    })
                    .fold(FileNameMapping::new(), |mut acc, (k, x)|{
                        acc.entry(k).or_insert(vec![]).push(x);
                        acc
                    });

    let mut grouping : FileNameGrouping 
                = mapping.into_iter()
                    .map(|(k, v)| {
                        let vsize = v.iter()
                            .map(|e| dirent_get_size(e))
                            .fold(0, |acc, num| acc + num);
                        
                        (k, vsize, v)
                    })
                    .collect();

    //  sort descending
    grouping.sort_by(|a, b| b.1.cmp(&a.1) );
    
    grouping
}

//-------------------------------------------------------------------------------------------------
fn main() {

    let args: Vec<String> = env::args().collect();
    let (dir2walk, pattern, skip_pattern, size_filter, verbose) = get_options(&args);
    
    let start = Instant::now();

    let file_re = Regex::new(format!(r"(?i){}$", pattern).as_ref()).unwrap();
    if verbose {
        println!("pattern regex is: {:#?}", file_re)
    }

    let is_skip_re_empty = skip_pattern.is_empty();
    let skip_re = Regex::new(format!(r"(?i){}$", skip_pattern).as_ref()).unwrap();
    if verbose {
        println!("filter regex is: {:#?}", skip_re)
    }

    let mut files : WalkDirEntryVec = WalkDir::new(dir2walk)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| if !is_skip_re_empty && is_filename_a_match(&e, &skip_re) {None} else {Some(e)})
            .filter_map(|e| if is_filename_a_match(&e, &file_re) {Some(e)} else {None})
            .collect();

    //  Sort descending, bigger files first
    files.sort_by(|a, b| compare_direntry(a, b) );

    let filename_grouping = get_filename_grouping(&files);

    let total_size = files
                        .iter()
                        .map(|e| dirent_get_size(e))
                        .fold(0, |acc, num| acc + num);

    let total_size_dups = filename_grouping
                            .iter()
                            .filter_map(|(_, vsize, val)| if val.len() < 2 {None} else {Some(vsize)})
                            .fold(0, |acc, num| acc + num);

    
    println!("found {} files in {} ms", files.len(), start.elapsed().as_millis());
    println!();
    println!("total size for {} files is         {:.3} MB", files.len(), to_mb(total_size));
    println!("total size for duplicated files is {:.3} MB", to_mb(total_size_dups));
    println!();

    for (key, vsize, val) in filename_grouping {

        if !verbose && val.len() < 2 || vsize < size_filter {
            continue;
        }

        println!("\n{} * {}, totalSize: {:.3}", key, val.len(), to_mb(vsize));
        println!("----------------------------------------");
        for v in val {
            println!("{:6.3}   {}", dirent_get_size_mb(v), v.path().to_string_lossy());
        }
    }

    println!();
}

