use std::io::{BufReader, Read, Write};
use std::fs::{File, OpenOptions};
use std::process::exit;
use std::env;
use args::{Args, ArgsError};
use getopts::Occur;
use serde_json::Value;
use regex::Regex;

const PROGRAM_DESC: &'static str = r#"Creates variables definition and
outputs definition from terraform code"#;
const PROGRAM_NAME: &'static str = "tfvarsauto";

fn parse(input: &Vec<String>) -> Result<(String, String, String), ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print the usage menu");
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

    args.parse(input)?;

    let help = args.value_of("help")?;
    if help {
        println!("{}", args.full_usage());
        exit(0);
    }
    let main_f = args.value_of("main")?;
    let vars_f = args.value_of("vars")?;
    let outputs_f = args.value_of("outputs")?;
    Ok((main_f, vars_f, outputs_f))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (main_file, vars_file, outputs_file) = match parse(&args) {
        Ok(d) => d,
        Err(e) => panic!("PANIC: {:?}", e),
    };

    println!(
        r#"Ok, let start with input file: {}
vars will be rewritten:{}
outputs will be rewritten too: {}"#,
        main_file,
        vars_file,
        outputs_file
    );

    let file = File::open(&main_file).unwrap();
    let mut br = BufReader::new(file);
    let mut main_tf = String::new();
    br.read_to_string(&mut main_tf).expect(
        "Unable to read from file main.tf",
    );
    let value: Value = hcl::from_str(&main_tf).unwrap();

    let mut vars_tf = OpenOptions::new()
        .append(true)
        .create(true) // Optionally create the file if it doesn't already exist
        .open(&vars_file)
        .expect("Unable to open file vars.tf");

    let mut outputs_tf = OpenOptions::new()
        .append(true)
        .create(true) // Optionally create the file if it doesn't already exist
        .open(&outputs_file)
        .expect("Unable to open file outputs.tf");

    let mut val_vec: Vec<Option<&serde_json::Map<std::string::String, serde_json::Value>>> =
        Vec::new();
    let mut vars_str = String::new();
    let mut vars_wrt: Vec<String> = Vec::new();
    let mut outputs_str = String::new();

    if value.is_object() {
        let ival = value.as_object();
        val_vec.push(ival);

        let re = Regex::new(r"var\.([[:word:]]+)").unwrap();

        while val_vec.len() > 0 {
            for (cur_str, cur_val) in val_vec.get(0).unwrap().unwrap() {
                let cvs = cur_val.to_string();
                if cur_val.is_string() {
                    for var_n in re.captures_iter(&cvs) {
                        let vs = var_n.get(1).unwrap().as_str().to_string();
                        if vars_wrt.contains(&vs) {
                            continue;
                        } else {
                            vars_str = format!(
                                r#"{}variable "{}" {{
  type        = string
  default     = null
  description = ""
}}
"#,
                                vars_str,
                                &vs
                            );
                            vars_wrt.push(vs);
                        };
                    }
                };
                if cur_val.is_object() {
                    let iv = cur_val.as_object();
                    let rvn = "resource".to_string();
                    let dvn = "data".to_string();
                    if [rvn, dvn].contains(&cur_str) {
                        let v = iv.clone();
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
                    };
                    val_vec.push(iv);
                };
            }
            val_vec.swap_remove(0);
        }
    };


    vars_tf.write_all(vars_str.as_bytes()).expect(
        "Unable to write data to vars.tf",
    );
    outputs_tf.write_all(outputs_str.as_bytes()).expect(
        "Unable to write data to outputs.tf",
    );
}
