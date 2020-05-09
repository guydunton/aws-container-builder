use regex::Regex;
use std::io;
use std::path::PathBuf;

pub struct DockerIgnore {
    rules: Vec<String>,
}

impl DockerIgnore {
    pub fn new(path: PathBuf) -> io::Result<DockerIgnore> {
        let contents = std::fs::read_to_string(path)?;

        let lines = contents.split("\n").collect();

        Ok(DockerIgnore {
            rules: Self::remove_comments(lines)
                .iter_mut()
                .map(|s| s.to_owned())
                .collect(),
        })
    }

    fn remove_comments(lines: Vec<&str>) -> Vec<&str> {
        lines
            .into_iter()
            .filter(|line| line.chars().nth(0) != Some('#'))
            .filter(|line| !line.trim().is_empty())
            .collect()
    }

    pub fn filter_files(&self, files: &Vec<PathBuf>) -> Vec<PathBuf> {
        files
            .iter()
            .filter(|file| !self.should_remove_file(file))
            .map(|path| path.clone())
            .collect::<Vec<PathBuf>>()
            .clone()
    }

    fn should_remove_file(&self, file: &PathBuf) -> bool {
        self.rules
            .iter()
            .map(|rule| {
                let s = file.to_str().unwrap();
                let matcher = Regex::new(rule).unwrap();
                matcher.is_match(s)
            })
            .any(|result| result)
    }
}

#[test]
fn remove_lines_works() {
    let lines = vec!["# first_line", "second_line", " "];

    assert_eq!(DockerIgnore::remove_comments(lines), vec!["second_line"]);
}

#[test]
fn filter_works_correctly() {
    use std::str::FromStr;

    let files = vec![
        "example_proj/dependencies/file",
        "example_proj/src/code.sh",
        "example_proj/Dockerfile",
    ];

    let paths: Vec<PathBuf> = files
        .iter()
        .map(|file| PathBuf::from_str(file).unwrap())
        .collect();

    let ignore = DockerIgnore {
        rules: vec!["dependencies".to_owned()],
    };

    let filtered = ignore.filter_files(&paths);

    let expected: Vec<PathBuf> = vec!["example_proj/src/code.sh", "example_proj/Dockerfile"]
        .into_iter()
        .map(|f| PathBuf::from_str(f).unwrap())
        .collect();

    assert_eq!(filtered, expected);
}
