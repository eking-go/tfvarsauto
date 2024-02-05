use args::{Args, ArgsError};
use getopts::Occur;
use regex::Regex;
use serde_json::Value;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::process::exit;

//=================================================================================
fn parse() -> Result<(String, String, String), ArgsError> {
    let input_args: Vec<String> = env::args().collect();
    let mut args = Args::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_DESCRIPTION"));
    args.flag("h", "help", "Print the usage menu");
    args.flag("", "version", "Print version");
    args.option(
        "m",
        "main",
        "The name of file for input with main terraform code, main.tf by default",
        "main.tf",
        Occur::Optional,
        Some(String::from("main.tf")),
    );
    args.option(
        "v",
        "vars",
        "The name of file to output with variables definition, vars.tf by default",
        "vars.tf",
        Occur::Optional,
        Some(String::from("vars.tf")),
    );
    args.option(
        "o",
        "outputs",
        "The name of file to output with outputs definition, outputs.tf by default",
        "outputs.tf",
        Occur::Optional,
        Some(String::from("outputs.tf")),
    );

    args.parse(input_args)?;

    let help = args.value_of("help")?;
    if help {
        println!("{}", args.full_usage());
        exit(0);
    }
    let version = args.value_of("version")?;
    if version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }
    let main_f = args.value_of("main")?;
    let vars_f = args.value_of("vars")?;
    let outputs_f = args.value_of("outputs")?;
    Ok((main_f, vars_f, outputs_f))
}

//=================================================================================
fn get_main_tf(main_file_name: String) -> Value {
    let file = File::open(&main_file_name);
    let file_r = match file {
        Ok(f) => f,
        Err(e) => panic!("Unable to open the file: {} {:?}", &main_file_name, e),
    };

    let mut br = BufReader::new(file_r);
    let mut main_tf_str = String::new();

    match br.read_to_string(&mut main_tf_str) {
        Ok(_) => match hcl::from_str(&main_tf_str) {
            Ok(val) => val,
            Err(ve) => panic!("Unable to parse the file: {} {:?}", &main_file_name, ve),
        },
        Err(e) => panic!("Unable to read the file: {} {:?}", &main_file_name, e),
    }
}

//=================================================================================
fn write_file(fname: String, string_contents: String) {
    let file = OpenOptions::new()
        .append(true)
        .create(true) // Optionally create the file if it doesn't already exist
        .open(&fname);

    let mut file_r = match file {
        Ok(f) => f,
        Err(e) => panic!("Unable to open the file: {} {:?}", &fname, e),
    };

    match file_r.write_all(string_contents.as_bytes()) {
        Ok(_) => (),
        Err(er) => panic!("Unable to write the file: {} {:?}", &fname, er),
    };
}
//=================================================================================
fn vars_text_file(value: Value) -> String {
    let mut val_vec: Vec<Option<&serde_json::Map<std::string::String, serde_json::Value>>> =
        Vec::new();
    let mut vars_str = String::new();
    let mut vars_vec: Vec<String> = Vec::new();

    if value.is_object() {
        let ival = value.as_object();
        val_vec.push(ival);

        let re = Regex::new(r"var\.([[:word:]]+)").unwrap();
        while val_vec.len() > 0 {
            for (_, cur_val) in val_vec.get(0).unwrap().unwrap() {
                if cur_val.is_string() {
                    let cvs = cur_val.to_string();
                    for var_n in re.captures_iter(&cvs) {
                        let vs = var_n.get(1).unwrap().as_str().to_string();
                        if vars_vec.contains(&vs) {
                            continue;
                        } else {
                            vars_vec.push(vs);
                        };
                    }
                } else if cur_val.is_object() {
                    let iv = cur_val.as_object();
                    val_vec.push(iv);
                };
            }
            val_vec.swap_remove(0);
        }
    };
    vars_vec.sort();
    for var in vars_vec {
        vars_str = format!(
            r#"{}variable "{}" {{
  type        = string
  default     = null
  description = ""
}}
"#,
            vars_str, &var
        );
    }
    vars_str
}

//=================================================================================
fn outputs_text_file(value: Value) -> String {
    let mut val_vec: Vec<Option<&serde_json::Map<std::string::String, serde_json::Value>>> =
        Vec::new();
    let mut outputs_str = String::new();

    if value.is_object() {
        let ival = value.as_object();
        val_vec.push(ival);

        while val_vec.len() > 0 {
            for (cur_str, cur_val) in val_vec.get(0).unwrap().unwrap() {
                if cur_val.is_object() {
                    let iv = cur_val.as_object();
                    let rvn = "resource".to_string();
                    let dvn = "data".to_string();
                    let lvn = "locals".to_string();
                    let mvn = "module".to_string();
                    let v = iv.clone();
                    if [dvn].contains(&cur_str) {
                        for (rst, rn) in v.unwrap() {
                            let sr = rn.clone();
                            for srn in sr.as_object().unwrap().keys() {
                                outputs_str = format!(
                                    r#"{os}output "{rt}_{rst}_{rn}_" {{
  value       = {rt}.{rst}.{rn}.
  description = ""
}}
"#,
                                    os = outputs_str,
                                    rt = &cur_str,
                                    rst = &rst,
                                    rn = &srn
                                );
                            }
                        }
                    } else if [rvn].contains(&cur_str) {
                        for (rst, rn) in v.unwrap() {
                            let sr = rn.clone();
                            for srn in sr.as_object().unwrap().keys() {
                                outputs_str = format!(
                                    r#"{os}output "{rst}_{rn}_" {{
  value       = {rst}.{rn}.
  description = ""
}}
"#,
                                    os = outputs_str,
                                    rst = &rst,
                                    rn = &srn
                                );
                            }
                        }
                    } else if [lvn, mvn].contains(&cur_str) {
                        for (rst, _) in v.unwrap() {
                            outputs_str = format!(
                                r#"{os}output "{rt}_{rst}_" {{
  value       = {rt}.{rst}.
  description = ""
}}
"#,
                                os = outputs_str,
                                rst = &rst,
                                rt = &cur_str
                            );
                        }
                    };
                    val_vec.push(iv);
                };
            }
            val_vec.swap_remove(0);
        }
    };
    outputs_str
}

//=================================================================================
fn main() {
    let (main_file, vars_file, outputs_file) = match parse() {
        Ok(d) => d,
        Err(e) => panic!("Unable to parse arguments: {:?}", e),
    };

    println!(
        r#"Ok, let start with input file: {}
vars will be rewritten: {}
outputs will be rewritten too: {}"#,
        main_file, vars_file, outputs_file
    );

    let value = get_main_tf(main_file);
    let vars_str = vars_text_file(value.clone());
    write_file(vars_file, vars_str);
    let outputs_str = outputs_text_file(value);
    write_file(outputs_file, outputs_str);
}
