#[async_trait::async_trait]
impl openapi::apis::board::Board<String> for crate::ApiImpl {
    async fn get_board(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        path_params: &openapi::models::GetBoardPathParams,
    ) -> Result<openapi::apis::board::GetBoardResponse, String> {
        Ok(
            match super::get_board::get_board(&self.board_events, path_params.id)? {
                Some(dsl) => {
                    openapi::apis::board::GetBoardResponse::Status200_TheCurrentContentOfTheBoard(
                        dsl,
                    )
                }
                None => openapi::apis::board::GetBoardResponse::Status404_NoBoardHasTheID(
                    openapi::models::Problem::new(
                        "about:blank".to_string(),
                        "Not Found".to_string(),
                        404,
                    ),
                ),
            },
        )
    }
}
