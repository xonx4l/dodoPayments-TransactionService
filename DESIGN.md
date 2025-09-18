# Transaction Service Design Specification

## Overview

This document outlines the design and architecture of the Transaction Service, a secure, scalable API for managing business accounts, processing financial transactions, and delivering webhooks.

## System Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Apps   │    │   Webhook       │    │   Monitoring    │
│                 │    │   Endpoints     │    │   (Jaeger)      │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          │                      │                      │
┌─────────▼───────┐    ┌─────────▼───────┐    ┌─────────▼───────┐
│   Load Balancer │    │   Transaction   │    │   Background    │
│   (Rate Limit)  │    │   Service       │    │   Workers       │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          │                      │                      │
┌─────────▼───────┐    ┌─────────▼───────┐    ┌─────────▼───────┐
│   API Gateway   │    │   PostgreSQL    │    │   Webhook       │
│   (Auth)        │    │   Database      │    │   Retry Queue   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **API Layer**: Axum-based REST API with authentication and rate limiting
2. **Service Layer**: Business logic for accounts, transactions, and webhooks
3. **Data Layer**: PostgreSQL with ACID transactions
4. **Webhook System**: Asynchronous delivery with retry logic
5. **Background Workers**: Webhook retry and cleanup tasks

## Data Model

### Database Schema

#### Accounts Table
```sql
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

#### API Keys Table
```sql
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE
);
```

#### Transactions Table
```sql
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    counterparty_account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    type VARCHAR(20) NOT NULL CHECK (type IN ('credit', 'debit', 'transfer')),
    amount BIGINT NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'failed', 'cancelled')),
    idempotency_key VARCHAR(255) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

#### Webhooks Table
```sql
CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    url VARCHAR(500) NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    secret VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

#### Webhook Deliveries Table
```sql
CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'failed', 'retrying')),
    response_status INTEGER,
    response_body TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

### Data Relationships

- **One-to-Many**: Account → API Keys
- **One-to-Many**: Account → Transactions
- **One-to-Many**: Account → Webhooks
- **One-to-Many**: Webhook → Webhook Deliveries
- **Many-to-One**: Transaction → Account (counterparty)

## API Design

### RESTful Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/health` | Health check | No |
| POST | `/api/v1/accounts` | Create account | No |
| GET | `/api/v1/accounts/{id}` | Get account | Yes |
| GET | `/api/v1/accounts/{id}/balance` | Get balance | Yes |
| POST | `/api/v1/transactions` | Create transaction | Yes |
| GET | `/api/v1/transactions/{id}` | Get transaction | Yes |
| POST | `/api/v1/webhooks` | Register webhook | Yes |
| GET | `/api/v1/webhooks/{id}` | Get webhook | Yes |
| PUT | `/api/v1/webhooks/{id}` | Update webhook | Yes |
| DELETE | `/api/v1/webhooks/{id}` | Delete webhook | Yes |

### Request/Response Format

All API requests and responses use JSON format with the following structure:

**Success Response:**
```json
{
  "data": { ... },
  "metadata": { ... }
}
```

**Error Response:**
```json
{
  "error": "Error message",
  "code": 400,
  "details": { ... }
}
```

## Security Design

### Authentication

- **API Key Authentication**: All endpoints (except account creation) require a valid API key
- **Key Storage**: API keys are hashed using SHA-256 before storage
- **Key Rotation**: Support for multiple active keys per account
- **Key Validation**: Keys are validated on every request

### Authorization

- **Account Isolation**: Users can only access their own account data
- **Transaction Authorization**: Users can only create transactions for their own account
- **Webhook Authorization**: Users can only manage their own webhooks

### Data Protection

- **Encryption at Rest**: Database encryption (handled by PostgreSQL)
- **Encryption in Transit**: HTTPS/TLS for all API communications
- **Webhook Signatures**: HMAC-SHA256 signatures for webhook payloads
- **Input Validation**: Comprehensive validation of all input data

## Transaction Processing

### Transaction Types

1. **Credit**: Add money to an account
2. **Debit**: Remove money from an account
3. **Transfer**: Move money between two accounts

### Atomicity Guarantees

- **Database Transactions**: All balance updates are wrapped in database transactions
- **ACID Compliance**: PostgreSQL ensures atomicity, consistency, isolation, and durability
- **Rollback on Failure**: Failed transactions are automatically rolled back

### Idempotency

- **Idempotency Keys**: Clients can provide unique keys to prevent duplicate transactions
- **Key Validation**: Duplicate keys return the original transaction instead of creating a new one
- **Key Storage**: Idempotency keys are stored with transactions for future lookups

## Webhook System

### Webhook Delivery

1. **Asynchronous Processing**: Webhooks are delivered asynchronously to avoid blocking transaction processing
2. **Event Filtering**: Webhooks only receive events they're subscribed to
3. **Signature Verification**: All webhook payloads include HMAC-SHA256 signatures
4. **Retry Logic**: Failed deliveries are retried with exponential backoff

### Retry Strategy

- **Maximum Attempts**: 3 attempts per webhook delivery
- **Retry Intervals**: 5 minutes, 15 minutes, 45 minutes
- **Exponential Backoff**: Increasing delays between retries
- **Dead Letter Queue**: Failed webhooks after max attempts are logged for manual review

### Webhook Security

- **Signature Verification**: Clients should verify webhook signatures using the provided secret
- **HTTPS Only**: Webhook URLs must use HTTPS
- **Secret Rotation**: Webhook secrets can be rotated without affecting existing deliveries

## Operational Considerations

### Monitoring and Observability

- **Structured Logging**: JSON-formatted logs with correlation IDs
- **OpenTelemetry Integration**: Distributed tracing with Jaeger
- **Health Checks**: Comprehensive health check endpoint
- **Metrics**: Request rates, error rates, and transaction volumes

### Scalability

- **Horizontal Scaling**: Stateless service design allows horizontal scaling
- **Database Connection Pooling**: Efficient database connection management
- **Async Processing**: Non-blocking webhook delivery
- **Rate Limiting**: Per-API-key rate limiting to prevent abuse

### Deployment

- **Docker Containerization**: Single container deployment
- **Docker Compose**: Local development environment
- **Database Migrations**: Automated schema migrations
- **Environment Configuration**: 12-factor app configuration

### Backup and Recovery

- **Database Backups**: Regular PostgreSQL backups
- **Point-in-Time Recovery**: PostgreSQL WAL-based recovery
- **Data Retention**: Configurable retention policies for webhook deliveries

## Trade-offs and Assumptions

### Design Decisions

1. **PostgreSQL over NoSQL**: Chosen for ACID compliance and relational data integrity
2. **Axum over Actix**: Chosen for better async performance and simpler middleware
3. **Synchronous API, Asynchronous Webhooks**: Balance between consistency and performance
4. **UUID Primary Keys**: Better for distributed systems and security
5. **BigInt for Balances**: Avoids floating-point precision issues

### Assumptions

1. **Single Currency**: All transactions are in USD (can be extended)
2. **Positive Balances**: Accounts can have negative balances (business decision)
3. **Immediate Settlement**: All transactions are settled immediately
4. **Webhook Reliability**: Clients are expected to handle webhook failures gracefully
5. **API Key Management**: Clients are responsible for secure key storage

### Limitations

1. **No Multi-Currency Support**: Currently only supports USD
2. **No Transaction Fees**: No built-in fee calculation
3. **No Audit Trail**: Limited audit logging (can be added)
4. **No Real-Time Notifications**: Only webhook-based notifications
5. **No Batch Operations**: All operations are individual (can be added)

## Future Enhancements

### Planned Features

1. **Multi-Currency Support**: Support for multiple currencies with exchange rates
2. **Transaction Fees**: Configurable fee calculation and collection
3. **Audit Logging**: Comprehensive audit trail for compliance
4. **Batch Operations**: Bulk transaction processing
5. **Real-Time Notifications**: WebSocket-based real-time updates
6. **Advanced Analytics**: Transaction analytics and reporting
7. **API Versioning**: Backward-compatible API versioning
8. **GraphQL API**: Alternative GraphQL endpoint

### Scalability Improvements

1. **Read Replicas**: Database read replicas for better performance
2. **Caching Layer**: Redis-based caching for frequently accessed data
3. **Message Queue**: Dedicated message queue for webhook delivery
4. **Microservices**: Split into separate services for accounts, transactions, and webhooks
5. **CDN Integration**: Static content delivery optimization

## Conclusion

This Transaction Service provides a solid foundation for financial transaction processing with strong security, reliability, and scalability features. The design prioritizes data consistency, security, and ease of use while maintaining flexibility for future enhancements.

The system is designed to handle moderate to high transaction volumes while providing reliable webhook delivery and comprehensive monitoring capabilities. The modular architecture allows for easy extension and modification as business requirements evolve.