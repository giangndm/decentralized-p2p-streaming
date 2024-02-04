use decentralized_p2p_streaming_web::add;
use protocol::P2pStreamRunner;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let runner = P2pStreamRunner::new(1.into());
            let value = add(*counter, 1);
            counter.set(value);
        }
    };

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <span>{ "Demo2" }</span>
            <p>{ *counter }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
