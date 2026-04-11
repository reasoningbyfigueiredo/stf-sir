use crate::model::Artifact;

pub fn to_yaml_string(artifact: &Artifact) -> Result<String, serde_yaml_ng::Error> {
    let yaml = serde_yaml_ng::to_string(artifact)?;
    let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml).to_owned();
    Ok(ensure_trailing_newline(yaml))
}

fn ensure_trailing_newline(mut yaml: String) -> String {
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    yaml
}
