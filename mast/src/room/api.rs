#[async_trait::async_trait]
impl openapi::apis::room::Room<String> for crate::ApiImpl {
    async fn get_room_boards(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        path_params: &openapi::models::GetRoomBoardsPathParams,
    ) -> Result<openapi::apis::room::GetRoomBoardsResponse, String> {
        Ok(
            match super::get_room_boards::get_room_boards(&self.room_events, path_params.id)? {
                Some(boards) => openapi::apis::room::GetRoomBoardsResponse::Status200_TheIDsOfTheBoardsAssociatedWithTheRoom(
                    openapi::models::GetRoomBoards200Response::new(boards),
                ),
                None => openapi::apis::room::GetRoomBoardsResponse::Status404_NoRoomHasTheID(
                    openapi::models::Problem::new(
                        "about:blank".to_string(),
                        "Not Found".to_string(),
                        404,
                    ),
                ),
            },
        )
    }

    async fn post_rooms(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        body: &openapi::models::PostRoomsRequest,
    ) -> Result<openapi::apis::room::PostRoomsResponse, String> {
        match super::create_room::create_room(&self.board_events, &self.room_events, &body.data) {
            Ok((id, share_url)) => Ok(
                openapi::apis::room::PostRoomsResponse::Status201_TheRoomWasCreated(
                    openapi::models::PostRooms201Response::new(id, share_url),
                ),
            ),
            Err(super::create_room::CreateRoomError::InvalidData(diagnostics)) => Ok(
                openapi::apis::room::PostRoomsResponse::Status400_TheDataCannotBeDecoded(
                    openapi::models::PostRooms400Response {
                        diagnostics: (!diagnostics.is_empty()).then(|| {
                            diagnostics
                                .iter()
                                .map(|d| {
                                    openapi::models::PostRooms400ResponseDiagnosticsInner::new(
                                        d.line as u32,
                                        d.to_string(),
                                    )
                                })
                                .collect()
                        }),
                        ..openapi::models::PostRooms400Response::new(
                            "about:blank".to_string(),
                            "Bad Request".to_string(),
                            400,
                        )
                    },
                ),
            ),
            Err(super::create_room::CreateRoomError::Internal(e)) => Err(e),
        }
    }
}
