
use uuid::Uuid;

use crate::{
    models::{AccountResponse, BalanceResponse, CreateAccountRequest, CreateAccountResponse},
}

pub async fn create_account( 
    State((account_service, _, _)): State<(AccountService, TransactionService )>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<CreateAccountResponse>> {
    let response = account_service.create_account(req).await?;
    Ok(Json(response))
}

pub async fn get_account(
    State((account_service, _, _)): State<(AccountService,TransactionService)>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<AccountResponse>> {
    let account = account_service.get_account(account_id).await?;
    Ok(Json(AccountResponse { account }))
}

pub async fn get_balance(
    State((account_service, _, _)): State<(AccountService, TransactionService)>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<BalanceResponse>> {
    let balance = account_service.get_balance(account_id).await?;
    Ok(Json(BalanceResponse {
        account_id,
        balance,
        currency: "USD".to_string(),
    }))
}

