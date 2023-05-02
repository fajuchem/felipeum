use std::rc::Rc;

use felipeum_signature::keypair::{new_keypair, Keypair};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde_json::json;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/portal.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="" view=  move |cx| view! { cx, <Home/> }/>
            </Routes>
        </Router>
    }
}

async fn post(to: String, value: String, keypair: Rc<Keypair>) -> String {
    let tx = json!({
        "from": hex::encode(keypair.public_key()),
        "to": to,
        "value": value.parse::<u64>().unwrap(),
        "nonce": 1,
    });

    log!("tx: {:?}", tx);
    let signature = keypair
        .sign_message(tx.to_string().as_bytes())
        .unwrap()
        .to_string();

    let body = json!({
        "jsonrpc": "2.0",
        "method": "sendTransaction",
        "id": 3,
        "params": [{
            "transaction": tx,
            "signature": signature
        }]
    })
    .to_string();

    log!("body: {:?}", body);

    let client = reqwest::Client::new();
    let result = client
        .post("http://127.0.0.1:4500")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    log!("{:?}", result);

    result
}

#[component]
fn Wallet(cx: Scope) -> impl IntoView {
    let keypair = Rc::new(new_keypair().unwrap());
    let (public_key, set_public_key) = create_signal(cx, hex::encode(keypair.public_key()));
    let (private_key, set_private_key) = create_signal(cx, hex::encode(keypair.secret()));

    let (to, set_to) = create_signal(cx, String::new());
    let (value, set_value) = create_signal(cx, String::new());

    let action = create_action(cx, |input: &(String, String, Rc<Keypair>)| {
        post(input.0.clone(), input.1.clone(), input.2.clone())
    });

    view! { cx,
        <div class="flex items-center justify-center h-screen text-center">
            <div>
                <div>{public_key}</div>
                <div>{private_key}</div>
                <div>
                <button
                    class="bg-amber-600 hover:bg-gray-400 px-5 py-3 text-white rounded-lg"
                    on:click=move |_| {
                        log!("start");
                        let new_keypair = new_keypair().unwrap();
                        set_private_key(hex::encode(new_keypair.public_key()));
                        set_public_key(hex::encode(new_keypair.secret()));
                        log!("end");
                }>"New Key"</button>
                </div>
                <div>
                    <input
                        class="border border-red-500 rounded "
                        type="text"
                        on:input=move |ev| set_to(event_target_value(&ev))
                        prop:value=to
                    />
                </div>
                <div>
                    <input type="number"
                        on:input=move |ev| set_value(event_target_value(&ev))
                        prop:value=value
                    />
                </div>
                <div>
                    <button
                        class="bg-amber-600 hover:bg-gray-400 px-5 py-3 text-white rounded-lg"
                        on:click=move |_| {
                            let key = keypair.clone();
                            log!("start");
                            action.dispatch((to.get(), value.get(), key));
                            log!("end");
                    }>"action"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    view! { cx,
        <main class="container mx-auto">
            <Wallet />
        </main>
    }
}
