fn operation_from_model(
    operation: &openapi::models::PostRoomOpsRequestOpsInner,
) -> Result<crate::board::operation::Operation, String> {
    Ok(match operation {
        openapi::models::PostRoomOpsRequestOpsInner::AddNote(add) => {
            crate::board::operation::Operation::AddNote {
                id: keel::note_id::NoteId::parse(&add.id)
                    .ok_or_else(|| format!("{} is not a note ID", add.id))?,
                note_type: keel::note_type::NoteType::parse(&add.note_type)
                    .ok_or_else(|| format!("{} is not a note type", add.note_type))?,
                text: add.text.clone(),
                x: add.x,
                y: add.y,
            }
        }
        openapi::models::PostRoomOpsRequestOpsInner::MoveNotes(moves) => {
            crate::board::operation::Operation::MoveNotes(
                crate::non_empty::NonEmpty::parse(
                    moves
                        .positions
                        .iter()
                        .map(|p| {
                            Ok(crate::board::operation::Position {
                                id: keel::note_id::NoteId::parse(&p.id)
                                    .ok_or_else(|| format!("{} is not a note ID", p.id))?,
                                x: p.x,
                                y: p.y,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                )
                .ok_or("positions is empty")?,
            )
        }
    })
}

#[async_trait::async_trait]
impl openapi::apis::room::Room<String> for crate::ApiImpl {
    async fn get_room_events(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        path_params: &openapi::models::GetRoomEventsPathParams,
    ) -> Result<openapi::apis::room::GetRoomEventsResponse, String> {
        let Some((snapshot, edited)) = super::get_room_events::get_room_events(
            &self.room_events,
            &self.board_events,
            &self.board_edited_events,
            &self.board_edited_senders,
            path_params.id,
        )?
        else {
            return Ok(
                openapi::apis::room::GetRoomEventsResponse::Status404_NoRoomHasTheID(
                    openapi::models::Problem::new(
                        "about:blank".to_string(),
                        "Not Found".to_string(),
                        404,
                    ),
                ),
            );
        };
        let snapshot = axum::response::sse::Event::default()
            .id(snapshot.seq.value().to_string())
            .event("snapshot")
            .json_data(openapi::models::BoardSnapshot::new(
                snapshot.seq.value(),
                snapshot.dsl,
            ))
            .map_err(|e| e.to_string())?;
        Ok(
            openapi::apis::room::GetRoomEventsResponse::Status200_TheStreamOfTheBoardOfTheRoom(
                Box::pin(futures_util::StreamExt::chain(
                    futures_util::stream::once(std::future::ready(Ok(snapshot))),
                    futures_util::StreamExt::scan(edited, (), |_, edited| {
                        std::future::ready(
                            axum::response::sse::Event::default()
                                .id(edited.seq().value().to_string())
                                .event("edited")
                                .json_data(openapi::models::BoardEdited::new(
                                    edited.seq().value(),
                                    edited
                                        .edits()
                                        .iter()
                                        .map(|edit| match edit.kind() {
                                            crate::board::operation::EditKind::NoteAdded {
                                                id,
                                                note_type,
                                                text,
                                                x,
                                                y,
                                            } => openapi::models::NoteAdded::new(
                                                "noteAdded".to_string(),
                                                note_type.as_str().to_string(),
                                                id.id().to_string(),
                                                text.clone(),
                                                *x,
                                                *y,
                                            )
                                            .into(),
                                            crate::board::operation::EditKind::NotesMoved(
                                                positions,
                                            ) => openapi::models::NotesMoved::new(
                                                "notesMoved".to_string(),
                                                positions
                                                    .iter()
                                                    .map(|p| {
                                                        openapi::models::MoveNotesPositionsInner::new(
                                                            p.id.to_string(),
                                                            p.x,
                                                            p.y,
                                                        )
                                                    })
                                                    .collect(),
                                            )
                                            .into(),
                                        })
                                        .collect(),
                                ))
                                .ok()
                                .map(Ok),
                        )
                    }),
                )),
            ),
        )
    }

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

    async fn post_room_ops(
        &self,
        _method: &axum::http::Method,
        _host: &axum_extra::headers::Host,
        _cookies: &axum_extra::extract::CookieJar,
        path_params: &openapi::models::PostRoomOpsPathParams,
        body: &openapi::models::PostRoomOpsRequest,
    ) -> Result<openapi::apis::room::PostRoomOpsResponse, String> {
        let operations = crate::non_empty::NonEmpty::parse(
            body.ops
                .iter()
                .map(operation_from_model)
                .collect::<Result<_, _>>()?,
        )
        .ok_or("ops is empty")?;
        match super::post_room_ops::post_room_ops(
            &self.room_events,
            &self.board_events,
            &self.board_edited_events,
            &self.board_edited_senders,
            path_params.id,
            operations,
        ) {
            Ok((seq, outcomes)) => Ok(
                openapi::apis::room::PostRoomOpsResponse::Status200_TheBatchWasApplied(
                    openapi::models::PostRoomOps200Response::new(
                        seq.value(),
                        outcomes
                            .into_iter()
                            .map(|outcome| match outcome {
                                crate::board::operation::Outcome::Applied => {
                                    openapi::models::PostRoomOps200ResponseResultsInner::new(
                                        "applied".to_string(),
                                    )
                                }
                                crate::board::operation::Outcome::Reassigned(id) => {
                                    openapi::models::PostRoomOps200ResponseResultsInner {
                                        id: Some(id.to_string()),
                                        ..openapi::models::PostRoomOps200ResponseResultsInner::new(
                                            "reassigned".to_string(),
                                        )
                                    }
                                }
                                crate::board::operation::Outcome::Skipped => {
                                    openapi::models::PostRoomOps200ResponseResultsInner::new(
                                        "skipped".to_string(),
                                    )
                                }
                            })
                            .collect(),
                    ),
                ),
            ),
            Err(super::post_room_ops::PostRoomOpsError::RoomNotFound) => Ok(
                openapi::apis::room::PostRoomOpsResponse::Status404_NoRoomHasTheID(
                    openapi::models::Problem::new(
                        "about:blank".to_string(),
                        "Not Found".to_string(),
                        404,
                    ),
                ),
            ),
            Err(super::post_room_ops::PostRoomOpsError::Internal(e)) => Err(e),
        }
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
