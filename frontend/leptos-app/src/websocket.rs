use server_fn::{codec::JsonEncoding, BoxedStream, ServerFnError, Websocket};
use leptos::*;
use futures::{SinkExt, StreamExt};

// WebSocket implementation using Leptos 0.8.0-alpha
#[server(protocol = Websocket<JsonEncoding, JsonEncoding>)]
async fn echo_websocket(
    input: BoxedStream<String, ServerFnError>,
) -> Result<BoxedStream<String, ServerFnError>, ServerFnError> {
    use futures::channel::mpsc;
    let mut input = input;

    // Create a channel of outgoing websocket messages
    let (mut tx, rx) = mpsc::channel(1);

    // Spawn a task to listen to the input stream of messages coming in over the websocket
    tokio::spawn(async move {
        while let Some(msg) = input.next().await {
            // Process message and send response
            tx.send(msg.map(|msg| msg.to_ascii_uppercase())).await;
        }
    });

    Ok(rx.into())
}

#[component]
pub fn WebSocketDemo() -> impl IntoView {
    use futures::channel::mpsc;
    
    let (mut tx, rx) = mpsc::channel(1);
    let latest = RwSignal::new(None);

    // Only listen for websocket messages on the client
    if cfg!(feature = "hydrate") {
        spawn_local(async move {
            match echo_websocket(rx.into()).await {
                Ok(mut messages) => {
                    while let Some(msg) = messages.next().await {
                        latest.set(Some(msg));
                    }
                }
                Err(e) => leptos::logging::warn!("{e}"),
            }
        });
    }

    view! {
        <div class="websocket-demo">
            <h2>"WebSocket Demo"</h2>
            <input type="text" on:input:target=move |ev| {
                tx.try_send(Ok(ev.target().value()));
            }/>
            <p>{latest}</p>
        </div>
    }
}