extern crate yaml_rust;
extern crate linked_hash_map;
extern crate clap;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::{yaml, YamlEmitter};
use clap::{App, Arg};

fn process_node(doc: &yaml::Yaml) -> yaml::Yaml {
    match *doc {
        yaml::Yaml::String(ref s) => {
            if !(s.starts_with("${") && s.ends_with("}")) {
                return yaml::Yaml::String(s.clone());
            }
            let key = s.trim_start_matches("${").trim_end_matches("}").trim();
            match env::var(key) {
                Ok(value) => yaml::Yaml::String(value),
                Err(_) => panic!("environment variable {} not set", s),
            }
        }
        yaml::Yaml::Array(ref arr) => {
            yaml::Yaml::Array(arr.iter().map(process_node).collect())
        }
        yaml::Yaml::Hash(ref hash) => {
            let mut result = linked_hash_map::LinkedHashMap::new();
            for (k, v) in hash {
                result.insert(k.clone(), process_node(v));
            }
            yaml::Yaml::Hash(result)
        }
        yaml::Yaml::Null => yaml::Yaml::Null,
        yaml::Yaml::BadValue => yaml::Yaml::BadValue,
        yaml::Yaml::Real(ref s) => yaml::Yaml::Real(s.clone()),
        yaml::Yaml::Integer(i) => yaml::Yaml::Integer(i), 
        yaml::Yaml::Boolean(b) => yaml::Yaml::Boolean(b),
        yaml::Yaml::Alias(u) => yaml::Yaml::Alias(u),
    }
}

fn main() {
    let matches = App::new("YamlFromEnv")
        .version("1.0")
        .author("Chinmay Kousik <chinmaykousik1@gmail.com>")
        .about("Replace values in yaml files from environment variables")
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .required(true)
            .help("path to input yaml file")
            .takes_value(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .help("path to output yaml file (default prints to stdout)")
            .takes_value(true))
        .get_matches();
    let input_file_path = matches.value_of("input").unwrap();
    let mut content = String::new();
    {
        let mut reader = File::open(&input_file_path).unwrap();
        reader.read_to_string(&mut content).unwrap();
    }

    let docs = yaml::YamlLoader::load_from_str(&content).unwrap();
    let mut writer: Box<Write> = match matches.value_of("output") {
        Some(path) => Box::new(std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .append(true)
            .open(path)
            .unwrap()),
        None => Box::new(std::io::stdout()),
    };
    for doc in &docs {
        let result = process_node(doc);
        let mut write_str = String::new();
        YamlEmitter::new(&mut write_str).dump(&result).unwrap();
        write!(&mut writer, "{}\n---", write_str.trim_start_matches("---")).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::process_node;
    use yaml_rust::{YamlEmitter, yaml::YamlLoader};

    #[test]
    fn test_process_node_no_replace() {
        let test_string: String = "
        key1:
            value: abc
        key2: def
        key3: [ 1, 2, 3]
        ".to_owned();
        let doc = YamlLoader::load_from_str(&test_string).unwrap();
        let result = process_node(&doc[0]);
        let mut result_string = String::new();
        YamlEmitter::new(&mut result_string).dump(&result).unwrap();
        println!("{:?}", result_string);
    }

    #[test]
    fn test_process_node_replace() {
        let test_string: String = String::from("key: ${HOME}");
        let doc = YamlLoader::load_from_str(&test_string).unwrap();
        let result = process_node(&doc[0]);
        assert_eq!(result["key"].as_str().unwrap(), std::env::var("HOME").unwrap());
    }
}
