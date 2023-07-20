use actix_web::{error, web, Error, HttpResponse};
use futures_util::StreamExt as _;
use mysql::Pool;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use super::{database, SignIn, SignInRes, SignOut, SelectTraders, Account, actions, Trade, Posr, NetWorthRe, IncomesRe, Equity, DateTrade, DelectOrders, AddOrders, AddPositions, UpdatePositions,AccountEquity, UpdateOriBalance, UpdateAlarms, AddAccounts, SelectId, SelectAccount};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status: u32,
    data: T,
}

pub async fn sign_in(
    mut payload: web::Payload,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SignIn>(&body)?;

    let query = database::check_account(db_pool.clone(), &obj.name, &obj.password);
    match query {
        Ok(data) => match data {
            Some(response) => {
                let rand_string: String = thread_rng()
                    .sample_iter(Alphanumeric)
                    .take(15)
                    .map(char::from)
                    .collect();
                match database::add_active(
                    db_pool,
                    response.acc_id,
                    &rand_string,
                    &response.acc_name,
                ) {
                    Ok(pros) => {
                        return Ok(HttpResponse::Ok().json(Response {
                            status: 200,
                            data: SignInRes {
                                name: response.acc_name,
                                account: response.acc_id,
                                admin: response.admin,
                                products: pros,
                                token: rand_string,
                            },
                        }));
                    }
                    Err(e) => {
                        return Err(error::ErrorNotFound(e));
                    }
                }
            }
            None => {
                return Err(error::ErrorNotFound("account not exist"));
            }
        },
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn sign_out(
    mut payload: web::Payload,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SignOut>(&body)?;

    match database::remove_active(db_pool.clone(), &obj.name, &obj.token) {
        Ok(()) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: format!("succeed"),
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取账户列表的权益杠杆率数据
pub async fn account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_traders(db_pool.clone()) {
        Ok(traders) => {
            let acct_re = actions::get_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


// 获取账户列表的权益杠杆率数据
pub async fn bybit_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_traders(db_pool.clone()) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_account_(traders).await;
            // println!("{:?}", acct_re);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取单个账户的详情数据
pub async fn single_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_one_traders(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_single_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


// 获取单个bybit账户的详情数据
pub async fn single_bybit_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_one_traders(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_account_(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取所有账户列表（显示为机器人列表）
pub async fn get_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectTraders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_all_traders(db_pool.clone(), &obj.account_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 获取权益数据（显示资金曲线）
pub async fn get_bybit_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_bybit_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 清除数据
pub async fn clear_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::clear_data(db_pool.clone());
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}




// 获取bian权益数据（显示资金曲线）
pub async fn get_bian_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_bian_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}



// 获取后续的bybit权益数据（显示资金曲线）
pub async fn get_total_bybit_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_total_bybit_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 获取bian权益数据（显示资金曲线）
pub async fn get_total_bian_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_total_bian_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}

pub async fn positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_history_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn futures_bybit_positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn spot_bybit_positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_spot_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_history_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn futures_bybit_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_futures_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn spot_bybit_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_spot_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn spot_bybit_usdc_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_usdc_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn assets(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_history_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}



pub async fn bybit_assets(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_history_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}



pub async fn incomes(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<IncomesRe>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_incomes(db_pool.clone()) {
        Ok(traders) => {
            let acc_income_re = actions::get_history_income(traders).await;
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acc_income_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn bybit_incomes(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_history_income(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_trades(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 获取bybit账户订单详细
pub async fn bybit_trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_bybit_trades(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


pub async fn history_incomes(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_incomes(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_income) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_income,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取账户权益
pub async fn pnl(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_equity(db_pool.clone());
    match data {
        Ok(histor_equity) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_equity,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 获取账户交易额
pub async fn is_price(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_trade_price(db_pool.clone());
    match data {
        Ok(histor_price) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_price,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取仓位数据
pub async fn position(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_positions(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_positions) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_positions,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 获取权益数据
pub async fn net_worth(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<NetWorthRe>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_networth(db_pool.clone());
    match data {
        Ok(histor_net_worths) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_net_worths,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 根据日期来获取账户的成交记录
pub async fn date_trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DateTrade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_history_trades(db_pool.clone(), &obj.start_time, &obj.end_time, &obj.tra_id);
    match data {
        Ok(histor_date_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_date_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 根据日期来获取bybit账户的成交记录
pub async fn date_bybit_trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DateTrade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_bybit_history_trades(db_pool.clone(), &obj.start_time, &obj.end_time, &obj.tra_id);
    match data {
        Ok(histor_date_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_date_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取所有的产品
pub async fn get_products_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_all_products(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 获取当前所有监控的挂单账户
pub async fn get_open_orders_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_alarm_open_orders(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 删除监控的挂单账户
pub async fn delect_open_orders_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delect_orders(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 添加监控的挂单账户
pub async fn add_open_orders_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_orders(db_pool.clone(), &obj.name, &obj.api_key, &obj.secret_key);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取当前所有监控的净头寸账户
pub async fn get_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_alarm_positions(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 删除监控的净头寸账户
pub async fn delect_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delect_positions(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 添加监控的净头寸账户
pub async fn add_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddPositions>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_positions(db_pool.clone(), &obj.name, &obj.api_key, &obj.secret_key, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 更新监控的净头寸账户阈值
pub async fn update_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdatePositions>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_positions(db_pool.clone(), &obj.name, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 更新账户份额
pub async fn update_ori_balance_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateOriBalance>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_ori_balance(db_pool.clone(), &obj.tra_id, &obj.ori_balance);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 更新账户是否进行监控
pub async fn update_accounts_alarm(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateAlarms>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_alarms(db_pool.clone(), &obj.name, &obj.alarm);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 删除账号
pub async fn delete_accounts_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delect_accounts(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 添加账号
pub async fn add_accounts_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddAccounts>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_accounts(db_pool.clone(), &obj.name, &obj.api_key, &obj.secret_key, &obj.alarm, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 查找tra_id，并添加到test_prod_tra中
pub async fn select_tra_id(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectId>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::select_id(db_pool.clone(), &obj.name, &obj.prod_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


//获取所有的净值数据
pub async fn get_net_worths_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_net_worths(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


//获取所有的权益数据
pub async fn get_equitys_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_equitys(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}