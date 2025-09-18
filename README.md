# Transaction Service

A secure, scalable transaction service built with Rust and Axum for managing business accounts, processing financial transactions, and delivering webhooks.

## Features

- 🔐 **API Authentication**: Secure access with API keys
- 💰 **Account Management**: Create accounts and check balances
- 💸 **Transaction Processing**: Credit, debit, and transfer operations with atomic updates
- 🔔 **Webhook System**: Reliable webhook delivery with retry logic
- 🗄️ **PostgreSQL Database**: ACID-compliant data storage
- 📚 **Comprehensive API Documentation**: Clear request/response formats
- 🐳 **Docker Compose**: One-command local setup
- 🔄 **Idempotency Support**: Prevent duplicate transactions
- ⚡ **Rate Limiting**: Per-API-key rate limiting
- 📊 **OpenTelemetry Integration**: Distributed tracing and metrics

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust 1.75+ (for local development)

### Using Docker Compose (Recommended)

1. **Clone and start the service:**
   ```bash
   git clone <repository-url>
   cd transaction-service
   docker-compose up --build
   ```

2. **The service will be available at:**
   - API: http://localhost:3000
   - Jaeger UI: http://localhost:16686
   - PostgreSQL: localhost:5432

### Local Development

1. **Install dependencies:**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install PostgreSQL (if not using Docker)
   # macOS: brew install postgresql
   # Ubuntu: sudo apt-get install postgresql postgresql-contrib
   ```

2. **Set up the database:**
   ```bash
   # Start PostgreSQL
   # macOS: brew services start postgresql
   # Ubuntu: sudo systemctl start postgresql

   # Create database
   createdb transaction_service
   ```

3. **Configure environment:**
   ```bash
   cp env.example .env
   # Edit .env with your database URL
   ```

4. **Run migrations and start the service:**
   ```bash
   cargo run
   ```

## API Usage Examples

### 1. Create an Account

```bash
curl -X POST http://localhost:3000/api/v1/accounts \
  -H "Content-Type: application/json" \
  -d '{
    "business_name": "Acme Corp",
    "email": "contact@acme.com"
  }'
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

### 2. Check Account Balance

```bash
curl -X GET http://localhost:3000/api/v1/accounts/123e4567-e89b-12d3-a456-426614174000/balance \
  -H "Authorization: Bearer your-api-key"
```

### 3. Create a Credit Transaction

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

### 4. Create a Transfer Transaction

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

### 5. Register a Webhook

```bash
curl -X POST http://localhost:3000/api/v1/webhooks \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-app.com/webhooks",
    "events": ["transaction.credit", "transaction.debit", "transaction.transfer"]
  }'
```

## Webhook Integration

### Webhook Payload

When a transaction occurs, your webhook endpoint will receive:

```json
{
  "event": "transaction.credit",
  "transaction": {
    "id": "789e0123-e89b-12d3-a456-426614174000",
    "account_id": "123e4567-e89b-12d3-a456-426614174000",
    "type": "credit",
    "amount": 1000,
    "description": "Payment received",
    "status": "completed",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "signature": "sha256=abc123..."
}
```

### Webhook Headers

- `X-Webhook-Signature`: HMAC-SHA256 signature for verification
- `X-Webhook-Event`: Event type (e.g., "transaction.credit")

### Signature Verification

```python
import hmac
import hashlib

def verify_webhook_signature(payload, signature, secret):
    expected_signature = hmac.new(
        secret.encode('utf-8'),
        payload.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()
    
    return hmac.compare_digest(f"sha256={expected_signature}", signature)
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:password@localhost:5432/transaction_service` |
| `PORT` | Server port | `3000` |
| `WEBHOOK_SECRET` | Webhook signature secret | `your-webhook-secret-key` |
| `JAEGER_ENDPOINT` | Jaeger tracing endpoint | `http://localhost:14268/api/traces` |
| `RUST_LOG` | Log level | `transaction_service=debug,tower_http=debug` |

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── error.rs             # Error handling
├── database.rs          # Database connection and migrations
├── models.rs            # Data models and DTOs
├── services/            # Business logic
│   ├── account.rs       # Account management
│   ├── transaction.rs   # Transaction processing
│   └── webhook.rs       # Webhook delivery
├── api/                 # HTTP handlers
│   ├── accounts.rs      # Account endpoints
│   ├── transactions.rs  # Transaction endpoints
│   ├── webhooks.rs      # Webhook endpoints
│   ├── auth.rs          # Authentication middleware
│   └── health.rs        # Health check
└── webhooks.rs          # Background webhook processing
```

### Running Tests

```bash
cargo test
```

### Database Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
cargo run --bin transaction-service
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Monitoring

### Health Check

```bash
curl http://localhost:3000/health
```

### Jaeger Tracing

Visit http://localhost:16686 to view distributed traces and performance metrics.

### Logs

The service outputs structured JSON logs with correlation IDs for easy debugging and monitoring.

## Security

- **API Keys**: All API requests require valid API keys
- **Input Validation**: Comprehensive validation of all input data
- **SQL Injection Protection**: Parameterized queries prevent SQL injection
- **Webhook Signatures**: HMAC-SHA256 signatures for webhook verification
- **Rate Limiting**: Per-API-key rate limiting to prevent abuse

## Performance

- **Async Processing**: Non-blocking webhook delivery
- **Connection Pooling**: Efficient database connection management
- **Idempotency**: Prevents duplicate transaction processing
- **Atomic Transactions**: Database-level ACID compliance

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions and support, please open an issue in the GitHub repository.