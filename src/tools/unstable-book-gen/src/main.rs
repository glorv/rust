// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Auto-generate stub docs for the unstable book

#![deny(warnings)]

extern crate tidy;

use tidy::features::{Feature, Features, collect_lib_features, collect_lang_features};
use tidy::unstable_book::{collect_unstable_feature_names, collect_unstable_book_section_file_names,
                          PATH_STR, LANG_FEATURES_DIR, LIB_FEATURES_DIR};
use std::collections::HashSet;
use std::io::Write;
use std::fs::{self, File};
use std::env;
use std::path::Path;

/// A helper macro to `unwrap` a result except also print out details like:
///
/// * The file/line of the panic
/// * The expression that failed
/// * The error itself
macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {}", stringify!($e), e),
    })
}

fn generate_stub_issue(path: &Path, name: &str, issue: u32) {
    let mut file = t!(File::create(path));
    t!(file.write_fmt(format_args!(include_str!("stub-issue.md"),
                                   name = name,
                                   issue = issue)));
}

fn generate_stub_no_issue(path: &Path, name: &str) {
    let mut file = t!(File::create(path));
    t!(file.write_fmt(format_args!(include_str!("stub-no-issue.md"),
                                   name = name)));
}

fn hset_to_summary_str(hset: HashSet<String>, dir: &str
) -> String {
    hset
        .iter()
        .map(|ref n| format!("    - [{}]({}/{}.md)",
                                      n,
                                      dir,
                                      n.replace('_', "-")))
        .fold("".to_owned(), |s, a| s + &a + "\n")
}

fn generate_summary(path: &Path, lang_features: &Features, lib_features: &Features) {
    let compiler_flags = collect_unstable_book_section_file_names(
        &path.join("compiler-flags"));

    let compiler_flags_str = hset_to_summary_str(compiler_flags,
                                                 "compiler-flags");

    let unstable_lang_features = collect_unstable_feature_names(&lang_features);
    let unstable_lib_features = collect_unstable_feature_names(&lib_features);

    let lang_features_str = hset_to_summary_str(unstable_lang_features,
                                                LANG_FEATURES_DIR);
    let lib_features_str = hset_to_summary_str(unstable_lib_features,
                                               LIB_FEATURES_DIR);

    let mut file = t!(File::create(&path.join("SUMMARY.md")));
    t!(file.write_fmt(format_args!(include_str!("SUMMARY.md"),
                                   compiler_flags = compiler_flags_str,
                                   language_features = lang_features_str,
                                   library_features = lib_features_str)));

}

fn has_valid_tracking_issue(f: &Feature) -> bool {
    if let Some(n) = f.tracking_issue {
        if n > 0 {
            return true;
        }
    }
    false
}

fn generate_unstable_book_files(src :&Path, out: &Path, features :&Features) {
    let unstable_features = collect_unstable_feature_names(features);
    let unstable_section_file_names = collect_unstable_book_section_file_names(src);
    t!(fs::create_dir_all(&out));
    for feature_name in &unstable_features - &unstable_section_file_names {
        let file_name = format!("{}.md", feature_name.replace('_', "-"));
        let out_file_path = out.join(&file_name);
        let feature = &features[&feature_name];

        if has_valid_tracking_issue(&feature) {
            generate_stub_issue(&out_file_path, &feature_name, feature.tracking_issue.unwrap());
        } else {
            generate_stub_no_issue(&out_file_path, &feature_name);
        }
    }
}

fn copy_recursive(path: &Path, to: &Path) {
    for entry in t!(fs::read_dir(path)) {
        let e = t!(entry);
        let t = t!(e.metadata());
        let dest = &to.join(e.file_name());
        if t.is_file() {
            t!(fs::copy(&e.path(), dest));
        } else if t.is_dir() {
            t!(fs::create_dir_all(dest));
            copy_recursive(&e.path(), dest);
        }
    }
}

fn main() {
    let src_path_str = env::args_os().skip(1).next().expect("source path required");
    let dest_path_str = env::args_os().skip(2).next().expect("destination path required");
    let src_path = Path::new(&src_path_str);
    let dest_path = Path::new(&dest_path_str).join("src");

    let lang_features = collect_lang_features(src_path);
    let mut bad = false;
    let lib_features = collect_lib_features(src_path, &mut bad, &lang_features);

    let doc_src_path = src_path.join(PATH_STR);

    t!(fs::create_dir_all(&dest_path));

    generate_unstable_book_files(&doc_src_path.join(LANG_FEATURES_DIR),
                                 &dest_path.join(LANG_FEATURES_DIR),
                                 &lang_features);
    generate_unstable_book_files(&doc_src_path.join(LIB_FEATURES_DIR),
                                 &dest_path.join(LIB_FEATURES_DIR),
                                 &lib_features);

    copy_recursive(&doc_src_path, &dest_path);

    generate_summary(&dest_path, &lang_features, &lib_features);
}