use poem::{listener::TcpListener, Route, Server};
use poem_openapi::OpenApiService;

use decentralized_p2p_streaming_native::rpc::ChordRpc;

#[tokio::main]
async fn main() {
    let api_service = OpenApiService::new(ChordRpc {}, "Chord RPC", "1.0").server("/chord");
    let ui = api_service.swagger_ui();

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(Route::new().nest("/chord", api_service).nest("/", ui))
        .await;
}
