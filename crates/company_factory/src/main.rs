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
                "Company sign-in also needs OIDC_ISSUER, OIDC_CLIENT_ID, OIDC_CLIENT_SECRET, and FACTORY_ALLOWED_DOMAIN. Google's issuer is https://accounts.google.com. Use a new web client and leave every existing client unchanged. No provider is bundled."
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
            "OIDC_ISSUER, OIDC_CLIENT_ID, OIDC_CLIENT_SECRET, and FACTORY_ALLOWED_DOMAIN are unset. /login returns 503 until they are set. Protected pages refuse everyone."
        );
    }
    let _ = tokio::signal::ctrl_c().await;
    server.shutdown().await;
}
