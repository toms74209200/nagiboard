#[async_trait::async_trait]
impl openapi::apis::room::Room<String> for crate::ApiImpl {
    async fn post_rooms(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        body: &openapi::models::PostRoomsRequest,
    ) -> Result<openapi::apis::room::PostRoomsResponse, String> {
        Ok(
            match super::create_room::create_room(&self.room_events, &body.data)? {
                Ok(id) => openapi::apis::room::PostRoomsResponse::Status201_TheRoomWasCreated(
                    openapi::models::PostRooms201Response::new(id),
                ),
                Err(diagnostics) => {
                    openapi::apis::room::PostRoomsResponse::Status422_TheDataCannotBeDecoded(
                        openapi::models::PostRooms422Response {
                            diagnostics: diagnostics.map(|diagnostics| {
                                diagnostics
                                    .iter()
                                    .map(|d| {
                                        openapi::models::PostRooms422ResponseDiagnosticsInner::new(
                                            d.line as u32,
                                            d.to_string(),
                                        )
                                    })
                                    .collect()
                            }),
                            ..openapi::models::PostRooms422Response::new(
                                "about:blank".to_string(),
                                "Unprocessable Content".to_string(),
                                422,
                            )
                        },
                    )
                }
            },
        )
    }
}
