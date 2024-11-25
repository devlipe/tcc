use tcc::{App, Output};

#[tokio::main]
async fn main() {
    Output::show_welcome_message();
    let context = tcc::AppContext::build_app_context_with_loading().await;
    let mut app = App::new(context);
    app.run();
}