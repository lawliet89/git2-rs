/*
 * libgit2 "clone" example
 *
 * Written by the libgit2 contributors
 *
 * To the extent possible under law, the author(s) have dedicated all copyright
 * and related and neighboring rights to this software to the public domain
 * worldwide. This software is distributed without any warranty.
 *
 * You should have received a copy of the CC0 Public Domain Dedication along
 * with this software. If not, see
 * <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

#![feature(old_orphan_check)]
#![deny(warnings)]

extern crate git2;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;

use docopt::Docopt;
use git2::{RemoteCallbacks, Progress};
use git2::build::{RepoBuilder, CheckoutBuilder};
use std::cell::RefCell;
use std::io::stdio;

#[derive(RustcDecodable)]
struct Args {
    arg_url: String,
    arg_path: String,
}

struct State {
    progress: Option<Progress<'static>>,
    total: uint,
    current: uint,
    path: Path,
    newline: bool,
}

fn print(state: &mut State) {
    let stats = state.progress.as_ref().unwrap();
    let network_pct = (100 * stats.received_objects()) / stats.total_objects();
    let index_pct = (100 * stats.indexed_objects()) / stats.total_objects();
    let co_pct = if state.total > 0 {
        (100 * state.current) / state.total
    } else {
        0
    };
    let kbytes = stats.received_bytes() / 1024;
    if stats.received_objects() == stats.total_objects() && false {
        if !state.newline {
            println!("");
            state.newline = true;
        }
        print!("Resolving deltas {}/{}\r", stats.indexed_deltas(),
               stats.total_deltas());
    } else {
        print!("net {:3}% ({:4} kb, {:5}/{:5})  /  idx {:3}% ({:5}/{:5})  \
                /  chk {:3}% ({:4}/{:4}) {}\r",
               network_pct, kbytes, stats.received_objects(),
               stats.total_objects(),
               index_pct, stats.indexed_objects(), stats.total_objects(),
               co_pct, state.current, state.total, state.path.display());
    }
    stdio::flush();
}

fn run(args: &Args) -> Result<(), git2::Error> {
    let state = RefCell::new(State {
        progress: None,
        total: 0,
        current: 0,
        path: Path::new("."),
        newline: false,
    });
    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        state.progress = Some(stats.to_owned());
        print(&mut *state);
        true
    });

    let mut co = CheckoutBuilder::new();
    co.progress(|path, cur, total| {
        let mut state = state.borrow_mut();
        state.path = Path::new(path);
        state.current = cur;
        state.total = total;
        print(&mut *state);
    });

    try!(RepoBuilder::new().remote_callbacks(cb).with_checkout(co)
                           .clone(args.arg_url.as_slice(),
                                  &Path::new(&args.arg_path)));
    println!("");

    Ok(())
}

fn main() {
    const USAGE: &'static str = "
usage: add [options] <url> <path>

Options:
    -h, --help          show this message
";

    let args = Docopt::new(USAGE).and_then(|d| d.decode())
                                 .unwrap_or_else(|e| e.exit());
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
