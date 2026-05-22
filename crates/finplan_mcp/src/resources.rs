use rmcp::model::{AnnotateAble, RawResource, Resource, ResourceContents};

use crate::schema_text;

fn make_resource(uri: &str, name: &str, description: &str, mime_type: &str) -> Resource {
    RawResource {
        uri: uri.into(),
        name: name.into(),
        title: None,
        description: Some(description.into()),
        mime_type: Some(mime_type.into()),
        size: None,
        icons: None,
        meta: None,
    }
    .no_annotation()
}

/// All available schema resources
pub fn list_resources() -> Vec<Resource> {
    vec![
        make_resource(
            "schema://full",
            "Full YAML Reference",
            "Complete scenario YAML reference documentation",
            "text/markdown",
        ),
        make_resource(
            "schema://accounts",
            "Account Types",
            "All 13 account types with fields and examples",
            "text/markdown",
        ),
        make_resource(
            "schema://events",
            "Event Structure",
            "Event structure: triggers, effects, once, enabled",
            "text/markdown",
        ),
        make_resource(
            "schema://triggers",
            "Trigger Types",
            "All trigger types with fields and YAML examples",
            "text/markdown",
        ),
        make_resource(
            "schema://effects",
            "Effect Types",
            "All effect types with fields and YAML examples",
            "text/markdown",
        ),
        make_resource(
            "schema://amounts",
            "Amount Types",
            "All amount types with nesting examples",
            "text/markdown",
        ),
        make_resource(
            "schema://parameters",
            "Parameters",
            "Parameter fields: dates, inflation, taxes, returns_mode",
            "text/markdown",
        ),
        make_resource(
            "schema://profiles",
            "Return Profiles",
            "Profile types (Fixed, Normal, etc.) and asset mappings",
            "text/markdown",
        ),
        make_resource(
            "schema://analysis",
            "Analysis Config",
            "MC iterations, sweep parameters, metrics, chart configs",
            "text/markdown",
        ),
        make_resource(
            "schema://example",
            "Example Scenario",
            "Complete example.yaml scenario file",
            "text/x-yaml",
        ),
        make_resource(
            "schema://patterns",
            "Common Patterns",
            "Reusable event pattern templates",
            "text/markdown",
        ),
    ]
}

/// Read a resource by URI and return its content
pub fn read_resource(uri: &str) -> Option<ResourceContents> {
    let text = match uri {
        "schema://full" => schema_text::full_schema(),
        "schema://accounts" => schema_text::accounts_schema(),
        "schema://events" => schema_text::events_schema(),
        "schema://triggers" => schema_text::triggers_schema(),
        "schema://effects" => schema_text::effects_schema(),
        "schema://amounts" => schema_text::amounts_schema(),
        "schema://parameters" => schema_text::parameters_schema(),
        "schema://profiles" => schema_text::profiles_schema(),
        "schema://analysis" => schema_text::analysis_schema(),
        "schema://example" => schema_text::example_yaml(),
        "schema://patterns" => schema_text::patterns_schema(),
        _ => return None,
    };

    Some(ResourceContents::text(text, uri))
}
