#[async_trait::async_trait]
impl openapi::apis::deck::Deck<String> for crate::ApiImpl {
    async fn get_deck(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        _query_params: &openapi::models::GetDeckQueryParams,
    ) -> Result<openapi::apis::deck::GetDeckResponse, String> {
        Ok(
            openapi::apis::deck::GetDeckResponse::Status200_ThePageOfTheBoardApp(
                super::get_deck::get_deck(&self.deck_dir)?,
            ),
        )
    }

    async fn get_deck_asset(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        path_params: &openapi::models::GetDeckAssetPathParams,
    ) -> Result<openapi::apis::deck::GetDeckAssetResponse, String> {
        Ok(
            match super::get_deck_asset::get_deck_asset(&self.deck_dir, &path_params.name)? {
                Some(script) => {
                    openapi::apis::deck::GetDeckAssetResponse::Status200_TheScript(script)
                }
                None => {
                    openapi::apis::deck::GetDeckAssetResponse::Status404_TheAppHasNoScriptOfTheName(
                        openapi::models::Problem::new(
                            "about:blank".to_string(),
                            "Not Found".to_string(),
                            404,
                        ),
                    )
                }
            },
        )
    }
}
