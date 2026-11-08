#[tokio::main]
async fn main() {
    let config = match company_factory::Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("AIFluxx company factory did not start: {error}");
            eprintln!(
                "Required settings: FACTORY_SESSION_SECRET (at least 32 bytes), FACTORY_PUBLIC_URL, FACTORY_REPO_ID, FACTORY_REPO_ROOT."
            );
            eprintln!(
                "Company sign-in also needs OIDC_ISSUER and OIDC_CLIENT_ID. OIDC_CLIENT_SECRET is required only when the identity provider uses a confidential client. No provider is bundled."
            );
            eprintln!("This process does not sign in to Warp and it is not Warp Factories.");
            std::process::exit(1);
        }
    };
    let sign_in_configured = config.oidc.is_some();
    let server = match company_factory::serve(config).await {
        Ok(server) => server,
        Err(error) => {
            eprintln!("AIFluxx company factory could not bind: {error}");
            std::process::exit(1);
        }
    };
    eprintln!(
        "AIFluxx company factory listening on http://{}",
        server.addr
    );
    eprintln!("This is not Warp Factories and it does not sign in to Warp.");
    if !sign_in_configured {
        eprintln!(
            "OIDC_ISSUER and OIDC_CLIENT_ID are unset. Protected pages refuse everyone until they are set."
        );
    }
    let _ = tokio::signal::ctrl_c().await;
    server.shutdown().await;
}
