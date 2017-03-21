extern crate argparse;

use argparse::ArgumentParser;
use std::process::{Command, Stdio};
use std::{fs, io};
use std::io::Write;
use std::path::{Path, PathBuf};


struct Options {
    editor: String,
    checker: String,
    filename: String,
    exit_success: i32,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            editor: "vim".to_string(),
            checker: "".to_string(),
            filename: "".to_string(),
            exit_success: 0,
        }
    }
}


enum EditorChoice {
    EditAgain,
    ExitWithoutSaving,
    QuitWithSaving,
}


fn editor_choice() -> EditorChoice {
    loop {
        print!("What now?: ");
        io::stdout().flush();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok().expect("Error reading input... :|");
        choice.pop();
        match choice.as_str() {
            "e" => return EditorChoice::EditAgain,
            "x" => return EditorChoice::ExitWithoutSaving,
            "Q" => return EditorChoice::QuitWithSaving,
            _   => {
                println!("Options are:\n\
                    \t(e)dit file again\n\
                    \te(x)it without saving changes to file\n\
                    \t(Q)uit and save changes to file (DANGER!)\n");
            },
        }
    }
}

fn main() {
    let mut options = Options::default();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("visudo-like safe editor.");
        parser.refer(&mut options.editor)
            .add_option(&["-e", "--editor"], argparse::Store, "Editor to use.")
            .envvar("EDITOR");
        parser.refer(&mut options.exit_success)
            .add_option(&["-s", "--success-code"], argparse::Store, "Exit code returned by the checker program on success.");
        parser.refer(&mut options.checker)
            .add_option(&["-c", "--checker"], argparse::Store, "Validator program.")
            .required();
        parser.refer(&mut options.filename)
            .add_argument("filename", argparse::Store, "File to edit.")
            .required();
        parser.parse_args_or_exit();
    }

    //Copy file to tmpfile
    let filepath = Path::new(options.filename.as_str());
    let filename = filepath.file_name().unwrap().to_str().unwrap();
    let parent_dir = filepath.parent().unwrap();
    let mut tmppath = PathBuf::from(parent_dir);
    tmppath.push(".".to_string() + filename + ".visafe");
    let tmppath_str = tmppath.into_os_string().into_string().unwrap();

    if filepath.exists() && filepath.is_dir() {
        println!("Path is directory...");
        return;
    }

    if filepath.exists() && filepath.is_file() {
        fs::copy(options.filename.as_str(), tmppath_str.clone())
            .expect("Could not copy file to temp file!");
    }

    let mut parse_ok = false;
    let mut done_editing = false;
    //Enter editing loop
    while !done_editing{
        let mut editor_cmd = Command::new(options.editor.as_str()).arg(tmppath_str.clone()).spawn()
        .expect("Could not launch editor!");
        editor_cmd.wait();
        //Check edited file
        let mut checker_cmd = Command::new(options.checker.as_str()).arg(tmppath_str.clone()).stdout(Stdio::piped())
            .spawn()
            .expect("Cloud not launch checker.");
        let checker_out = checker_cmd.wait_with_output().expect("Failed to capture checker!");
        if checker_out.status.code().unwrap() == options.exit_success {
            parse_ok = true;
            done_editing = true;
        } else {
            println!("The file contains errors: \n{}", String::from_utf8(checker_out.stdout).unwrap());
            match editor_choice() {
                EditorChoice::EditAgain => done_editing = false,
                EditorChoice::ExitWithoutSaving => done_editing = true,
                EditorChoice::QuitWithSaving => { done_editing = true; parse_ok = true; },
            }
        }
    }

    //Replace original file with edited one or cleanup tmp file
    if Path::new(tmppath_str.as_str()).is_file() {
        if parse_ok {
            fs::rename(tmppath_str.clone(), filepath).expect("Could not put file in place!");
        } else {
            fs::remove_file(tmppath_str.clone());
        }
    }
}
