# textminer-rs - PII Redaction Service

A high-performance, privacy-focused microservice for redacting Personally Identifiable Information (PII) from text using BERT-based Named Entity Recognition (NER). Built with Rust and Actix Web for maximum performance and reliability.

## Features

- 🔍 **Advanced PII Detection**: Uses BERT-based NER to identify various types of PII including:
  - Person names
  - Locations
  - Email addresses
  - Organizations
  - And more...

- ⚡ **High Performance**: Built with Rust and Actix-Web for exceptional throughput and low latency
- 🎯 **Simple API**: Easy-to-use HTTP endpoints for integration with any application
- 🔄 **Concurrent Processing**: Handles multiple requests efficiently with async/await
- 🏗️ **Production-Ready**: Includes proper error handling and health checks
- 🚀 **Efficient Model Loading**: Lazy initialization of the BERT model for faster startup

## Getting Started

### Prerequisites

- Rust (latest stable version)
- libtorch (required by rust-bert)

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/k5602/textminer-rs.git
   cd textminer-rs
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

### Running the Service

```bash
cargo run --release
```

The service will start on `http://0.0.0.0:8080`

## API Endpoints

### Redact Text

Detect and redact PII from the provided text.

- **URL**: `/api/redact`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:

  ```json
  {
      "text": "John Doe lives in New York and works at Example Corp. Contact: john@example.com"
  }
  ```

- **Success Response**:
  - **Code**: 200 OK
  - **Content**:

    ```json
    {
        "redacted_text": "[PERSON] lives in [LOCATION] and works at [ORG]. Contact: [EMAIL]",
        "processing_time_ms": 42
    }
    ```

### Health Check

Check if the service is running and the BERT model is loaded.

- **URL**: `/api/health`
- **Method**: `GET`
- **Success Response**:
  - **Code**: 200 OK
  - **Content**:
    ```json
    {
        "status": "ok",
        "model_loaded": true
    }
    ```

## Performance

The service is optimized for high throughput and low latency, with the NER model loaded just once at startup and shared across requests using thread-safe reference counting. On average, it processes text in under 100ms, though the first request may take longer as it loads the BERT model into memory.

## Web Interface

Access the web interface at `http://localhost:8080` to test the redaction service interactively.


## Development

### Building for Development

```bash
cargo build## Running with Docker

```bash
# Build the Docker image
docker build -t redactr .

# Run the container
docker run -p 8080:8080 redactr
```
```

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- [rust-bert](https://github.com/guillaume-be/rust-bert) - Rust native BERT implementation
- [Actix-Web](https://actix.rs/) - Powerful, pragmatic, and extremely fast web framework for Rust


Made with ❤️ in Rust by [Khaled](https://github.com/k5602)