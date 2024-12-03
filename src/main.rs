use tcc::{App, AppContext, Output};

#[tokio::main]
async fn main() {
    Output::show_welcome_message();
    static CONTEXT: AppContext = AppContext::build_app_context_with_loading().await;
    let mut app = App::new(&CONTEXT);
    app.run();
}