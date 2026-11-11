use std::collections::BTreeMap;

use ndc_models;

use crate::{arguments::check_all_arguments_used, query::Result, state::Row};

pub(crate) fn collection_info() -> ndc_models::CollectionInfo {
    ndc_models::CollectionInfo {
        name: "streaming_error".into(),
        description: Some("A collection for testing streaming errors".into()),
        collection_type: "streaming_error".into(),
        arguments: BTreeMap::new(),
        uniqueness_constraints: BTreeMap::from_iter([(
            "StreamingErrorByID".into(),
            ndc_models::UniquenessConstraint {
                unique_columns: vec!["id".into()],
            },
        )]),
        relational_mutations: None,
    }
}

pub(crate) fn rows(
    arguments: &BTreeMap<ndc_models::ArgumentName, serde_json::Value>,
    _state: &crate::state::AppState,
) -> Result<Vec<Row>> {
    check_all_arguments_used(arguments)?;
    Ok(vec![BTreeMap::from_iter([(
        "id".into(),
        serde_json::Value::Number(1.into()),
    )])])
}
