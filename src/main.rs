use tcc::{App, AppContext};

#[tokio::main]
async fn main() {
    let context = AppContext::my_app_context().await;
    let mut app = App::new(context);
    app.run();
}

