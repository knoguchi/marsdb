use std::collections::BTreeMap;

use crate::error::GraphError;
use crate::model::PropertyValue;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct NodeRecord {
    pub label_id: u32,
    pub props: BTreeMap<String, PropertyValue>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct EdgeRecord {
    pub label_id: u32,
    pub src: u64,
    pub dst: u64,
    pub props: BTreeMap<String, PropertyValue>,
}

pub(crate) fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, GraphError> {
    Ok(postcard::to_allocvec(value)?)
}

pub(crate) fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, GraphError> {
    Ok(postcard::from_bytes(bytes)?)
}
