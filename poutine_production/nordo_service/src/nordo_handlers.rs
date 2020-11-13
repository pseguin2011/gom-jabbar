use std::time::{Duration, SystemTime};
use warp::http::StatusCode;
use warp::reply::json;
use warp::{Rejection, Reply};

use crate::nordo_models::{
    BoilRequest, BoiledPotatoesResponse, BoilingErrorResponse, BoilingState, BoilingStatus,
    BoilingStatusResponse,
};

const BOIL_TIME: u64 = 900;

pub struct NordoHandlers;

impl NordoHandlers {
    /// Starts boiling the potatoes
    ///
    /// ## Returns
    /// The success message of an http status code of 200
    pub async fn start_boiling_potatoes(
        request: BoilRequest,
        state: BoilingState,
    ) -> Result<impl Reply, Rejection> {
        let mut state = state.write().await;
        if let Some(time) = state.time {
            if time.elapsed().unwrap() < Duration::new(BOIL_TIME, 0) {
                let res = BoilingErrorResponse {
                    error: "There are already potatoes boiling.".into(),
                };
                return Ok(warp::reply::with_status(
                    json(&res),
                    StatusCode::PRECONDITION_FAILED,
                ));
            }
        }
        println!("Message From Nordo Service: Starting to boil potatoes");
        state.time = Some(SystemTime::now());
        state.potatoes = Some(request.potatoes);
        Ok(warp::reply::with_status(
            json(&String::from("")),
            StatusCode::OK,
        ))
    }

    /// Returns the status of currently boiling potatoes
    ///
    /// ## Returns
    /// The status of the potatoes as a response
    pub async fn get_potatoes_status(state: BoilingState) -> Result<impl Reply, Rejection> {
        if let Some(time) = state.read().await.time {
            let time = if let Ok(time) = time.elapsed() {
                time
            } else {
                return Ok(warp::reply::with_status(
                    json(&String::from(
                        "Could not unwrap time elapsed for boiling potatoes",
                    )),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            };

            let status = if time > Duration::new(BOIL_TIME, 0) {
                BoilingStatus::LikeButter
            } else if time > Duration::new(BOIL_TIME / 5 * 4, 0) {
                BoilingStatus::ReasonablySoft
            } else if time > Duration::new(BOIL_TIME / 5 * 3, 0) {
                BoilingStatus::StartingToSoften
            } else if time > Duration::new(BOIL_TIME / 5 * 2, 0) {
                BoilingStatus::StillWouldntEatIt
            } else {
                BoilingStatus::HardAsAPotatoRock
            };

            println!(
                "Message From Nordo Service: The potatoes boiling status is now {:?}",
                status
            );
            return Ok(warp::reply::with_status(
                json(&BoilingStatusResponse { status }),
                StatusCode::OK,
            ));
        } else {
            return Ok(warp::reply::with_status(
                json(&String::from("Nothing is boiling at the moment")),
                StatusCode::OK,
            ));
        }
    }

    /// If the potatoes are boiled, they are then returned, otherwise the status
    /// of the potatoes is returned
    ///
    /// ## Returns
    /// The boiled potatoes or a message giving them their status
    pub async fn get_boiled_potatoes(state: BoilingState) -> Result<impl Reply, Rejection> {
        Self::get_boiled_potatoes_helper(state, BOIL_TIME).await
    }

    /// Helper function for getting boiled potatoes
    /// If the potatoes are boiled, they are then returned, otherwise the status
    /// of the potatoes is returned
    ///
    /// ## Returns
    /// The boiled potatoes or a message giving them their status
    async fn get_boiled_potatoes_helper(
        state: BoilingState,
        boil_time: u64,
    ) -> Result<impl Reply, Rejection> {
        let state = state.read().await;
        if let (Some(mut potatoes), Some(time)) = (state.potatoes.clone(), state.time) {
            let time = if let Ok(time) = time.elapsed() {
                time
            } else {
                return Ok(warp::reply::with_status(
                    json(&String::from(
                        "Could not unwrap time elapsed for boiling potatoes",
                    )),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            };
            if time < Duration::new(boil_time, 0) {
                return Ok(warp::reply::with_status(
                    json(&String::from("The potatoes have not finished boiling")),
                    StatusCode::OK,
                ));
            }

            println!("Message From Nordo Service: Sending boiled potatoes");
            potatoes.iter_mut().for_each(|mut p| p.boiled = true);
            return Ok(warp::reply::with_status(
                json(&BoiledPotatoesResponse { potatoes }),
                StatusCode::OK,
            ));
        } else {
            return Ok(warp::reply::with_status(
                json(&String::from("Nothing is boiling at the moment")),
                StatusCode::OK,
            ));
        }
    }
}

impl shared::NotifyMontroyashi for NordoHandlers {
    fn get_robot_name() -> &'static str {
        "Nordo Service"
    }
}

impl shared::TemperatureManagement for NordoHandlers {}

#[tokio::test]
async fn boil_potatoes_with_status() {
    let state = std::sync::Arc::new(tokio::sync::RwLock::new(crate::nordo_models::Boiling {
        time: None,
        potatoes: None,
    }));

    assert!(NordoHandlers::start_boiling_potatoes(
        BoilRequest {
            potatoes: vec![shared::Potato {
                size: 9,
                oil_used: None,
                boiled: false,
                coated_in_maple_syrup: false,
                fried: false,
            }],
        },
        state.clone(),
    )
    .await
    .is_ok());

    assert!(state.read().await.potatoes.is_some());

    assert!(NordoHandlers::get_boiled_potatoes_helper(state.clone(), 0)
        .await
        .is_ok());
    assert!(state.read().await.potatoes.is_none());
}
