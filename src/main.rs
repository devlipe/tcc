use tokio::sync::watch;
use tcc::{App, Output};

#[tokio::main]
async fn main() {
    Output::show_welcome_message();
    let context = tcc::AppContext::build_app_context_with_loading().await;
    let mut app = App::new(context);
    app.run();
}

async fn show_loading()
{
        let (tx , rx)= watch::channel(true);
        // Spawn the loading animation as a background task
        let animation_handle = tokio::spawn(Output::loading_animation(rx));

        // Signal the animation to stop
        let _ = tx.send(false);

        // Wait for the animation task to finish
        animation_handle.await.unwrap();
}
