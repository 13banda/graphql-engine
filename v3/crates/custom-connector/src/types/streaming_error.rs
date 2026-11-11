use std::collections::BTreeMap;

pub(crate) fn definition() -> ndc_models::ObjectType {
    ndc_models::ObjectType {
        description: Some("A type for testing streaming errors".into()),
        fields: BTreeMap::from_iter([(
            "id".into(),
            ndc_models::ObjectField {
                description: Some("The primary key".into()),
                r#type: ndc_models::Type::Named { name: "Int".into() },
                arguments: BTreeMap::new(),
            },
        )]),
        foreign_keys: BTreeMap::new(),
    }
}
