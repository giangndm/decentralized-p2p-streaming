use poem_openapi::{param::Query, payload::Json, Object, OpenApi, OpenApiService};

#[derive(Debug, Object)]
pub struct SuccessorInfo {
    pub node_id: u32,
    pub address: String,
}

pub struct ChordRpc {}

#[OpenApi]
impl ChordRpc {
    #[oai(path = "/successor", method = "get")]
    async fn successor(&self, key: Query<u32>) -> Json<SuccessorInfo> {
        todo!("")
    }
}
