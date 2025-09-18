# Transaction Service API Documentation

## Overview

The Transaction Service provides a secure, reliable API for managing business accounts, processing transactions, and delivering webhooks. All API endpoints require authentication via API keys.

**Base URL:** `http://localhost:3000/api/v1`

## Authentication

All API requests must include an API key in the Authorization header:

```
Authorization: Bearer <your-api-key>
```

API keys are generated when creating an account and provide access to all account-related operations.

## Endpoints

### Health Check

#### GET /health

Check if the service is running.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "service": "transaction-service"
}
```

### Accounts

#### POST /api/v1/accounts

Create a new business account.

**Request Body:**
```json
{
  "business_name": "Acme Corp",
  "email": "contact@acme.com"
}
```

**Response:**
```json
{
  "account": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "business_name": "Acme Corp",
    "email": "contact@acme.com",
    "balance": 0,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "api_key": "your-api-key-here"
}
```

#### GET /api/v1/accounts/{account_id}

Get account details.

**Response:**
```json
{
  "account": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "business_name": "Acme Corp",
    "email": "contact@acme.com",
    "balance": 10000,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### GET /api/v1/accounts/{account_id}/balance

Get account balance.

**Response:**
```json
{
  "account_id": "123e4567-e89b-12d3-a456-426614174000",
  "balance": 10000,
  "currency": "USD"
}
```

### Transactions

#### POST /api/v1/transactions

Create a new transaction.

**Request Body:**
```json
{
  "idempotency_key": "unique-key-123",
  "type": "credit",
  "amount": 1000,
  "description": "Payment received",
  "counterparty_account_id": "456e7890-e89b-12d3-a456-426614174000"
}
```

**Transaction Types:**
- `credit`: Add money to account
- `debit`: Remove money from account
- `transfer`: Move money between accounts (requires `counterparty_account_id`)

**Response:**
```json
{
  "transaction": {
    "id": "789e0123-e89b-12d3-a456-426614174000",
    "account_id": "123e4567-e89b-12d3-a456-426614174000",
    "counterparty_account_id": "456e7890-e89b-12d3-a456-426614174000",
    "type": "credit",
    "amount": 1000,
    "description": "Payment received",
    "status": "completed",
    "idempotency_key": "unique-key-123",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### GET /api/v1/transactions/{transaction_id}

Get transaction details.

**Response:**
```json
{
  "transaction": {
    "id": "789e0123-e89b-12d3-a456-426614174000",
    "account_id": "123e4567-e89b-12d3-a456-426614174000",
    "counterparty_account_id": "456e7890-e89b-12d3-a456-426614174000",
    "type": "credit",
    "amount": 1000,
    "description": "Payment received",
    "status": "completed",
    "idempotency_key": "unique-key-123",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### Webhooks

#### POST /api/v1/webhooks

Register a webhook endpoint.

**Request Body:**
```json
{
  "url": "https://your-app.com/webhooks",
  "events": ["transaction.credit", "transaction.debit", "transaction.transfer"]
}
```

**Response:**
```json
{
  "webhook": {
    "id": "webhook-123",
    "account_id": "123e4567-e89b-12d3-a456-426614174000",
    "url": "https://your-app.com/webhooks",
    "events": ["transaction.credit", "transaction.debit", "transaction.transfer"],
    "secret": "webhook-secret",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### GET /api/v1/webhooks/{webhook_id}

Get webhook details.

#### PUT /api/v1/webhooks/{webhook_id}

Update webhook configuration.

#### DELETE /api/v1/webhooks/{webhook_id}

Delete webhook.

## Webhook Payload

When a transaction occurs, webhooks receive the following payload:

```json
{
  "event": "transaction.credit",
  "transaction": {
    "id": "789e0123-e89b-12d3-a456-426614174000",
    "account_id": "123e4567-e89b-12d3-a456-426614174000",
    "counterparty_account_id": "456e7890-e89b-12d3-a456-426614174000",
    "type": "credit",
    "amount": 1000,
    "description": "Payment received",
    "status": "completed",
    "idempotency_key": "unique-key-123",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "signature": "sha256=abc123..."
}
```

**Headers:**
- `X-Webhook-Signature`: HMAC-SHA256 signature for verification
- `X-Webhook-Event`: Event type (e.g., "transaction.credit")

## Error Responses

All errors follow this format:

```json
{
  "error": "Error message",
  "code": 400
}
```

**Common Error Codes:**
- `400`: Bad Request (validation errors, insufficient funds)
- `401`: Unauthorized (invalid API key)
- `404`: Not Found (account, transaction, or webhook not found)
- `409`: Conflict (idempotency key already used)
- `429`: Too Many Requests (rate limit exceeded)
- `500`: Internal Server Error

## Rate Limiting

API requests are rate limited per API key:
- 1000 requests per hour
- 100 requests per minute

Rate limit headers are included in responses:
- `X-RateLimit-Limit`: Request limit per window
- `X-RateLimit-Remaining`: Remaining requests in current window
- `X-RateLimit-Reset`: Time when the rate limit resets

## Idempotency

Transaction creation supports idempotency keys to prevent duplicate transactions. Include an `idempotency_key` in your request, and if the same key is used again, the original transaction will be returned instead of creating a new one.

## Example Usage

### 1. Create Account
```bash
curl -X POST http://localhost:3000/api/v1/accounts \
  -H "Content-Type: application/json" \
  -d '{
    "business_name": "Acme Corp",
    "email": "contact@acme.com"
  }'
```

### 2. Create Credit Transaction
```bash
curl -X POST http://localhost:3000/api/v1/transactions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "credit-123",
    "type": "credit",
    "amount": 1000,
    "description": "Initial deposit"
  }'
```

### 3. Create Transfer Transaction
```bash
curl -X POST http://localhost:3000/api/v1/transactions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "transfer-123",
    "type": "transfer",
    "amount": 500,
    "description": "Payment to supplier",
    "counterparty_account_id": "456e7890-e89b-12d3-a456-426614174000"
  }'
```

### 4. Register Webhook
```bash
curl -X POST http://localhost:3000/api/v1/webhooks \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-app.com/webhooks",
    "events": ["transaction.credit", "transaction.debit", "transaction.transfer"]
  }'
```