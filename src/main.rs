use anyhow::Result;
use secret_book_server::{
    configurations::get_configuration, log::init_subscriber, startup::Application,
};

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = init_subscriber("app".into(), "info".into());

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).expect("Failed to build application");
    application.run_until_stopped().await?;
    Ok(())
}
